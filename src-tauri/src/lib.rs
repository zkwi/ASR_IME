mod aliyun_asr;
mod app_log;
mod asr;
mod asr_activity;
mod asr_provider;
mod asr_ws;
mod audio;
mod autostart;
mod commands;
mod config;
mod config_validation;
mod error;
mod hotkey;
mod hotword_generator;
mod hotword_history;
mod llm_client;
mod llm_endpoint;
mod llm_post_edit;
mod llm_request_adapter;
mod main_window;
mod overlay;
mod protocol;
mod screen_context;
mod session;
mod setup_guide;
mod stats;
mod system_audio;
mod text_output;
mod tray;
mod update;

use serde::Serialize;
use session::SessionController;
use tauri::{Emitter, Manager, WindowEvent};

const APP_WINDOW_ICON: tauri::image::Image<'static> = tauri::include_image!("./icons/128x128.png");

#[derive(Clone, Serialize)]
struct CloseToTrayRequest {
    first_time: bool,
    behavior: String,
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info
            .location()
            .map(|loc| format!("{}:{}", loc.file(), loc.line()))
            .unwrap_or_else(|| "unknown location".to_string());
        let payload = panic_info
            .payload()
            .downcast_ref::<&str>()
            .map(|value| (*value).to_string())
            .or_else(|| {
                panic_info
                    .payload()
                    .downcast_ref::<String>()
                    .map(|value| value.to_string())
            })
            .unwrap_or_else(|| "unknown panic payload".to_string());
        app_log::warn(format!("panic at {}: {}", location, payload));
    }));

    if let Err(err) = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            app_log::info("检测到重复启动，已唤起现有主窗口。");
            main_window::show_existing(app, "重复启动");
        }))
        .plugin(tauri_plugin_opener::init())
        .manage(SessionController::default())
        .invoke_handler(tauri::generate_handler![
            commands::config_commands::get_app_snapshot,
            commands::config_commands::get_setup_status,
            commands::config_commands::load_app_config,
            commands::config_commands::get_config_migration_candidate,
            commands::config_commands::migrate_config_to_default_path,
            commands::config_commands::save_app_config,
            commands::config_commands::test_asr_config,
            commands::config_commands::test_llm_config,
            commands::config_commands::test_screen_context,
            commands::config_commands::open_setup_guide,
            commands::config_commands::open_doubao_asr_docs,
            commands::config_commands::open_aliyun_asr_docs,
            commands::diagnostic_commands::open_log_file,
            commands::diagnostic_commands::get_diagnostic_report,
            commands::diagnostic_commands::copy_diagnostic_report_to_clipboard,
            commands::diagnostic_commands::copy_recent_input_text_to_clipboard,
            commands::config_commands::set_tray_language,
            commands::diagnostic_commands::hide_main_window,
            commands::diagnostic_commands::set_config_exit_guard,
            commands::diagnostic_commands::exit_application,
            commands::config_commands::update_close_preference,
            commands::diagnostic_commands::log_frontend_error,
            commands::diagnostic_commands::log_frontend_event,
            commands::diagnostic_commands::get_usage_stats,
            commands::diagnostic_commands::get_local_data_status,
            commands::diagnostic_commands::clear_usage_stats,
            commands::diagnostic_commands::clear_recent_context,
            commands::diagnostic_commands::get_auto_hotword_status,
            commands::diagnostic_commands::clear_hotword_history,
            commands::diagnostic_commands::generate_hotword_candidates,
            commands::update_commands::check_for_update,
            commands::update_commands::download_and_install_update,
            commands::session_commands::get_overlay_text,
            commands::session_commands::get_overlay_config,
            commands::session_commands::list_audio_input_devices,
            commands::session_commands::get_session_state,
            commands::session_commands::start_recording,
            commands::session_commands::stop_recording,
            commands::session_commands::toggle_recording
        ])
        .setup(|app| {
            app_log::info(format!(
                "VoxType Tauri client started. version={}",
                env!("CARGO_PKG_VERSION")
            ));
            if let Some(window) = app.get_webview_window("main") {
                if let Err(err) = window.set_icon(APP_WINDOW_ICON.clone()) {
                    app_log::warn(format!("设置主窗口图标失败: {}", err));
                }
            }
            app_log::info("startup stage: create overlay begin");
            let _ = overlay::create_overlay_window(app.handle());
            app_log::info("startup stage: create overlay done");
            app_log::info("startup stage: setup tray begin");
            if let Err(err) = tray::setup_tray(app.handle()) {
                app_log::warn(err);
            }
            app_log::info("startup stage: setup tray done");
            app_log::info("startup stage: startup message begin");
            tray::show_startup_message();
            app_log::info("startup stage: startup message done");
            app_log::info("startup stage: setup guide check begin");
            setup_guide::open_if_config_missing(app.handle());
            app_log::info("startup stage: setup guide check done");
            if let Ok(loaded) = config::load_config() {
                apply_autostart_in_background(loaded.data.startup);
            }
            app_log::info("startup stage: global hotkey thread start");
            hotkey::start_global_hotkey_thread(app.handle().clone());
            app_log::info("startup stage: input hook thread start");
            hotkey::start_input_hook_thread(app.handle().clone());
            app_log::info("startup stage: setup complete");
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }

            if let WindowEvent::CloseRequested { api, .. } = event {
                let close_config = config::load_config()
                    .map(|loaded| {
                        (
                            normalize_close_behavior(&loaded.data.tray.close_behavior).to_string(),
                            loaded.data.tray.close_to_tray_notice_shown,
                        )
                    })
                    .unwrap_or_else(|err| {
                        app_log::warn(format!("读取关闭行为配置失败，默认隐藏到托盘: {}", err));
                        ("close_to_tray".to_string(), true)
                    });
                if main_window::request_config_exit_guard(window.app_handle(), "window_close") {
                    api.prevent_close();
                    app_log::info("配置保存失败，已在关闭前请求用户确认。");
                    return;
                }
                if close_config.0 == "direct_exit" {
                    app_log::info("关闭主窗口触发直接退出。");
                    tray::exit_app(window.app_handle());
                    return;
                }

                api.prevent_close();
                let should_ask = close_config.0 == "ask_every_time" || !close_config.1;
                if should_ask {
                    let _ = window.show();
                    let _ = window.set_focus();
                    if let Err(err) = window.emit(
                        "close-to-tray-requested",
                        CloseToTrayRequest {
                            first_time: !close_config.1,
                            behavior: close_config.0,
                        },
                    ) {
                        app_log::warn(format!("发送关闭到托盘提示事件失败: {}", err));
                    } else {
                        app_log::info("已提示用户主窗口将隐藏到托盘。");
                    }
                } else if let Err(err) = window.hide() {
                    app_log::warn(format!("隐藏主窗口失败: {}", err));
                } else {
                    let _ = window.emit("main-window-hidden", ());
                    app_log::info("主窗口已隐藏到托盘。");
                }
            }
        })
        .run(tauri::generate_context!())
    {
        app_log::warn(format!("Tauri application exited with error: {}", err));
    }
}

fn normalize_close_behavior(value: &str) -> &str {
    match value {
        "direct_exit" => "direct_exit",
        "ask_every_time" => "ask_every_time",
        _ => "close_to_tray",
    }
}

fn apply_autostart_in_background(startup: config::StartupConfig) {
    std::thread::spawn(move || {
        if let Err(err) = autostart::apply(&startup) {
            app_log::warn(format!("配置已保存，但开机自启动后台同步失败: {}", err));
        }
    });
}
