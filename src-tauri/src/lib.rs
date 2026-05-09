mod app_log;
mod asr;
mod asr_ws;
mod audio;
mod autostart;
mod config;
mod config_validation;
mod error;
mod hotkey;
mod hotword_generator;
mod hotword_history;
mod llm_post_edit;
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

use config::{AppConfig, LoadedConfig};
use serde::Serialize;
use session::SessionController;
use stats::StatsSnapshot;
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

const APP_WINDOW_ICON: tauri::image::Image<'static> = tauri::include_image!("./icons/128x128.png");

#[derive(Serialize)]
struct AppSnapshot {
    hotkey: String,
    current_version: String,
}

#[derive(Debug, Serialize)]
struct SetupStatus {
    ready: bool,
    missing_auth: bool,
    has_audio_device: bool,
    hotkey: String,
    paste_method: String,
    privacy_recent_context_enabled: bool,
    warnings: Vec<SetupWarning>,
}

#[derive(Debug, Serialize)]
struct SetupWarning {
    code: String,
    level: String,
    title: String,
    message: String,
    action: String,
}

#[derive(Serialize)]
struct ConnectionTestResult {
    message: String,
}

#[derive(Serialize)]
struct LocalDataStatus {
    recent_context_enabled: bool,
    recent_context_count: usize,
    auto_hotwords_enabled: bool,
    auto_hotword_entry_count: usize,
    auto_hotword_total_chars: usize,
    stats_event_count: usize,
    screen_context_enabled: bool,
    llm_post_edit_enabled: bool,
    restore_clipboard_after_paste: bool,
}

#[derive(Serialize)]
struct ConfigSaveError {
    message: String,
    errors: Vec<config::ConfigValidationError>,
}

#[derive(Serialize)]
struct DiagnosticReport {
    text: String,
}

#[derive(Clone, Serialize)]
struct CloseToTrayRequest {
    first_time: bool,
    behavior: String,
}

#[tauri::command]
fn get_app_snapshot() -> Result<AppSnapshot, String> {
    let loaded = config::load_config()?;

    Ok(AppSnapshot {
        hotkey: loaded.data.hotkey,
        current_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tauri::command]
fn get_setup_status() -> Result<SetupStatus, String> {
    let loaded = config::load_config()?;
    let has_audio_device = audio::list_input_devices()
        .map(|devices| !devices.is_empty())
        .unwrap_or(false);

    Ok(build_setup_status(loaded.data, has_audio_device))
}

fn build_setup_status(data: AppConfig, has_audio_device: bool) -> SetupStatus {
    let missing_auth =
        data.auth.app_key.trim().is_empty() || data.auth.access_key.trim().is_empty();
    let mut warnings = Vec::new();

    if missing_auth {
        warnings.push(SetupWarning {
            code: "ASR_AUTH_MISSING".to_string(),
            level: "blocking".to_string(),
            title: "ASR 密钥未填写".to_string(),
            message: "填写 App Key 和 Access Key 后才能开始语音识别。".to_string(),
            action: "asr_auth".to_string(),
        });
    }
    if !has_audio_device {
        warnings.push(SetupWarning {
            code: "MIC_DEVICE_NOT_FOUND".to_string(),
            level: "blocking".to_string(),
            title: "未检测到麦克风".to_string(),
            message: "请接入或启用麦克风，然后重新检查设备。".to_string(),
            action: "audio".to_string(),
        });
    }
    let any_trigger_enabled = data.triggers.hotkey_enabled
        || data.triggers.middle_mouse_enabled
        || data.triggers.right_alt_enabled;
    if !any_trigger_enabled {
        warnings.push(SetupWarning {
            code: "TRIGGER_DISABLED".to_string(),
            level: "blocking".to_string(),
            title: "触发方式未开启".to_string(),
            message: "请至少开启主快捷键、右 Alt 或鼠标中键中的一种。".to_string(),
            action: "hotkey".to_string(),
        });
    }
    SetupStatus {
        ready: !missing_auth && has_audio_device && any_trigger_enabled,
        missing_auth,
        has_audio_device,
        hotkey: data.hotkey,
        paste_method: data.typing.paste_method,
        privacy_recent_context_enabled: data.context.enable_recent_context,
        warnings,
    }
}

#[tauri::command]
fn load_app_config() -> Result<LoadedConfig, String> {
    match config::load_config() {
        Ok(loaded) => {
            app_log::info(format!("配置加载完成: exists={}", loaded.exists));
            Ok(loaded)
        }
        Err(err) => {
            app_log::warn(format!("配置加载失败: {}", err));
            Err(err)
        }
    }
}

#[tauri::command]
fn save_app_config(app: AppHandle, config: AppConfig) -> Result<LoadedConfig, ConfigSaveError> {
    let previous_config = config::load_config().ok().map(|loaded| loaded.data);
    if let Err(errors) = config::validate_config(&config) {
        let validation_error_count = errors.len();
        let blocking_errors = blocking_validation_errors(errors, previous_config.as_ref(), &config);
        if !blocking_errors.is_empty() {
            app_log::warn(format!(
                "配置保存失败: validation_errors={}",
                blocking_errors.len()
            ));
            return Err(ConfigSaveError {
                message: "配置存在不合法字段，请修改后再保存。".to_string(),
                errors: blocking_errors,
            });
        }
        app_log::warn(format!(
            "配置存在未改动的隐藏高级字段错误，已保留原值并继续保存: validation_errors={}",
            validation_error_count
        ));
    }
    let side_effects = config_side_effects(previous_config.as_ref(), &config);
    if hotkey_registration_test_needed(previous_config.as_ref(), &config) {
        if let Err(err) = hotkey::can_register_hotkey(&config.hotkey) {
            app_log::warn(format!(
                "配置保存失败: hotkey register test failed: {}",
                err
            ));
            return Err(ConfigSaveError {
                message: format!("快捷键注册测试失败：{}", err),
                errors: vec![config::ConfigValidationError {
                    field: "hotkey".to_string(),
                    message: "该快捷键可能已被其他程序占用，请换一个。".to_string(),
                }],
            });
        }
    }
    match config::save_config(config) {
        Ok(loaded) => {
            apply_config_side_effects(app.clone(), &loaded, side_effects);
            app_log::info(format!(
                "配置保存完成: hotkey_enabled={}, middle_mouse_enabled={}, right_alt_enabled={}, hotkey_restart_scheduled={}, launch_on_startup={}, autostart_sync_scheduled={}, update_auto_check={}, update_repo={}, llm_enabled={}, close_behavior={}",
                loaded.data.triggers.hotkey_enabled,
                loaded.data.triggers.middle_mouse_enabled,
                loaded.data.triggers.right_alt_enabled,
                side_effects.restart_hotkey,
                loaded.data.startup.launch_on_startup,
                side_effects.apply_autostart,
                loaded.data.update.auto_check_on_startup,
                loaded.data.update.github_repo,
                loaded.data.llm_post_edit.enabled,
                loaded.data.tray.close_behavior
            ));
            Ok(loaded)
        }
        Err(err) => {
            app_log::warn(format!("配置保存失败: {}", err));
            Err(ConfigSaveError {
                message: err,
                errors: Vec::new(),
            })
        }
    }
}

fn blocking_validation_errors(
    errors: Vec<config::ConfigValidationError>,
    previous_config: Option<&AppConfig>,
    next_config: &AppConfig,
) -> Vec<config::ConfigValidationError> {
    errors
        .into_iter()
        .filter(|error| {
            !unchanged_hidden_config_field(previous_config, next_config, error.field.as_str())
        })
        .collect()
}

fn unchanged_hidden_config_field(
    previous_config: Option<&AppConfig>,
    next_config: &AppConfig,
    field: &str,
) -> bool {
    let Some(previous_config) = previous_config else {
        return false;
    };
    match field {
        "audio.sample_rate" => previous_config.audio.sample_rate == next_config.audio.sample_rate,
        "audio.channels" => previous_config.audio.channels == next_config.audio.channels,
        "audio.segment_ms" => previous_config.audio.segment_ms == next_config.audio.segment_ms,
        "audio.max_record_seconds" => {
            previous_config.audio.max_record_seconds == next_config.audio.max_record_seconds
        }
        "audio.stop_grace_ms" => {
            previous_config.audio.stop_grace_ms == next_config.audio.stop_grace_ms
        }
        "audio.silence_auto_stop_seconds" => {
            previous_config.audio.silence_auto_stop_seconds
                == next_config.audio.silence_auto_stop_seconds
        }
        "audio.silence_level_threshold" => {
            previous_config.audio.silence_level_threshold
                == next_config.audio.silence_level_threshold
        }
        "request.final_result_timeout_seconds" => {
            previous_config.request.final_result_timeout_seconds
                == next_config.request.final_result_timeout_seconds
        }
        "llm_post_edit.timeout_seconds" => {
            previous_config.llm_post_edit.timeout_seconds
                == next_config.llm_post_edit.timeout_seconds
        }
        "ui.width" => previous_config.ui.width == next_config.ui.width,
        "ui.height" => previous_config.ui.height == next_config.ui.height,
        "ui.background_color" => {
            previous_config.ui.background_color == next_config.ui.background_color
        }
        "ui.text_color" => previous_config.ui.text_color == next_config.ui.text_color,
        "typing.paste_delay_ms" => {
            previous_config.typing.paste_delay_ms == next_config.typing.paste_delay_ms
        }
        "typing.clipboard_snapshot_max_bytes" => {
            previous_config.typing.clipboard_snapshot_max_bytes
                == next_config.typing.clipboard_snapshot_max_bytes
        }
        "request.ws_url" => previous_config.request.ws_url == next_config.request.ws_url,
        "update.github_repo" => {
            previous_config.update.github_repo == next_config.update.github_repo
        }
        "auto_hotwords.max_history_chars" => {
            previous_config.auto_hotwords.max_history_chars
                == next_config.auto_hotwords.max_history_chars
        }
        "auto_hotwords.max_candidates" => {
            previous_config.auto_hotwords.max_candidates == next_config.auto_hotwords.max_candidates
        }
        "screen_context.max_chars" => {
            previous_config.screen_context.max_chars == next_config.screen_context.max_chars
        }
        "screen_context.timeout_ms" => {
            previous_config.screen_context.timeout_ms == next_config.screen_context.timeout_ms
        }
        _ => false,
    }
}

#[tauri::command]
async fn test_asr_config(config: AppConfig) -> Result<ConnectionTestResult, String> {
    app_log::info("用户开始测试豆包 ASR 配置。");
    match asr_ws::test_connection(&config).await {
        Ok(()) => {
            app_log::info("豆包 ASR 配置测试成功。");
            Ok(ConnectionTestResult {
                message: "豆包 ASR 测试成功，当前 Key 可用。".to_string(),
            })
        }
        Err(err) => {
            app_log::warn(format!("豆包 ASR 配置测试失败: {}", err));
            Err(err)
        }
    }
}

#[tauri::command]
async fn test_llm_config(config: AppConfig) -> Result<ConnectionTestResult, String> {
    app_log::info("用户开始测试大模型配置。");
    match llm_post_edit::test_connection(&config).await {
        Ok(()) => {
            app_log::info("大模型配置测试成功。");
            Ok(ConnectionTestResult {
                message: "大模型测试成功，当前 API Key 可用。".to_string(),
            })
        }
        Err(err) => {
            app_log::warn(format!("大模型配置测试失败: {}", err));
            Err(err)
        }
    }
}

#[tauri::command]
fn test_screen_context(
    config: AppConfig,
) -> Result<screen_context::ScreenContextTestResult, String> {
    app_log::info("用户开始测试屏幕 OCR 上下文。");
    match screen_context::test_current_window_on_worker(&config.screen_context) {
        Ok(result) => {
            app_log::info(format!(
                "屏幕 OCR 上下文测试完成: chars={}, elapsed_ms={}, language={}, image={}x{}",
                result.text_chars,
                result.elapsed_ms,
                result.selected_language.as_deref().unwrap_or("unknown"),
                result.image_width,
                result.image_height
            ));
            Ok(result)
        }
        Err(err) => {
            app_log::warn(format!("屏幕 OCR 上下文测试失败: {}", err));
            Err(err)
        }
    }
}

#[tauri::command]
fn open_setup_guide(app: AppHandle) -> Result<(), String> {
    app_log::info("用户打开配置指南。");
    setup_guide::open(&app).map_err(|err| {
        app_log::warn(format!("打开配置指南失败: {}", err));
        err
    })
}

#[tauri::command]
fn open_log_file(app: AppHandle) -> Result<(), String> {
    match tray::open_log_file_from_main(&app) {
        Ok(()) => Ok(()),
        Err(err) => {
            app_log::warn(err.clone());
            Err(err)
        }
    }
}

#[tauri::command]
fn get_diagnostic_report(
    session: State<'_, SessionController>,
) -> Result<DiagnosticReport, String> {
    let report = build_diagnostic_report(&session)?;
    app_log::info("用户生成诊断报告。");
    Ok(report)
}

#[tauri::command]
fn copy_diagnostic_report_to_clipboard(
    session: State<'_, SessionController>,
) -> Result<DiagnosticReport, String> {
    let report = build_diagnostic_report(&session)?;
    text_output::copy_text_to_clipboard(&report.text)?;
    app_log::info("用户复制诊断报告到剪贴板。");
    Ok(report)
}

#[tauri::command]
fn copy_recent_input_text_to_clipboard(text: String) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("没有可复制的识别文本。".to_string());
    }
    text_output::copy_text_to_clipboard(&text)
}

#[tauri::command]
fn set_tray_language(app: AppHandle, language: String) -> Result<(), String> {
    tray::set_language(&app, &language)
}

fn build_diagnostic_report(
    session: &State<'_, SessionController>,
) -> Result<DiagnosticReport, String> {
    let loaded = config::load_config()?;
    let state = session.current_state();
    let asr_ready = !loaded.data.auth.app_key.trim().is_empty()
        && !loaded.data.auth.access_key.trim().is_empty();
    let trigger_summary = enabled_trigger_summary(&loaded.data);
    let recent_error = state.error_code.as_deref().unwrap_or("无");
    let recent_context_summary = if loaded.data.context.enable_recent_context {
        format!("已启用，保存条数 {}", config::recent_context_count())
    } else {
        "未启用".to_string()
    };
    let auto_hotword_summary = hotword_history::status()
        .map(|status| {
            if status.enabled {
                format!(
                    "已启用，保存条数 {}，约 {} 字",
                    status.entry_count, status.total_chars
                )
            } else {
                "未启用".to_string()
            }
        })
        .unwrap_or_else(|_| "状态读取失败".to_string());
    let text = format!(
        "VoxType 诊断报告\n\
版本: {}\n\
系统: {} / {}\n\
配置文件: {} ({})\n\
日志文件: {}\n\
ASR 已配置: {}\n\
LLM 润色: {}\n\
最近上下文: {}\n\
自动热词候选: {}\n\
触发方式: {}\n\
最近会话状态: {:?}\n\
最近错误码: {}\n\
诊断报告内容: 不包含识别正文、屏幕 OCR 正文、热词、Prompt、最近上下文正文、自动热词历史正文、候选词、密钥原文\n\
日志脱敏范围: key/token/bearer/password/secret 类字段和本机用户路径\n",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        redact_user_path(&loaded.path),
        if loaded.exists {
            "已存在"
        } else {
            "未创建"
        },
        redact_user_path(&app_log::log_path().display().to_string()),
        if asr_ready { "是" } else { "否" },
        if loaded.data.llm_post_edit.enabled {
            "已启用"
        } else {
            "未启用"
        },
        recent_context_summary,
        auto_hotword_summary,
        trigger_summary,
        state.phase,
        recent_error
    );
    Ok(DiagnosticReport { text })
}

#[tauri::command]
fn hide_main_window(app: AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("main") else {
        return Err("找不到主窗口。".to_string());
    };
    window
        .hide()
        .map_err(|err| format!("隐藏主窗口失败: {}", err))?;
    app_log::info("主窗口已隐藏到托盘。");
    Ok(())
}

#[tauri::command]
fn exit_application(app: AppHandle) {
    app_log::info("用户从主窗口退出程序。");
    tray::exit_app(&app);
}

#[tauri::command]
fn update_close_preference(
    close_behavior: Option<String>,
    close_to_tray_notice_shown: Option<bool>,
) -> Result<LoadedConfig, String> {
    let mut loaded = config::load_config()?;
    if let Some(behavior) = close_behavior {
        loaded.data.tray.close_behavior = normalize_close_behavior(&behavior).to_string();
    }
    if let Some(shown) = close_to_tray_notice_shown {
        loaded.data.tray.close_to_tray_notice_shown = shown;
    }
    let saved = config::save_config(loaded.data)?;
    app_log::info(format!(
        "关闭行为配置已更新: close_behavior={}, notice_shown={}",
        saved.data.tray.close_behavior, saved.data.tray.close_to_tray_notice_shown
    ));
    Ok(saved)
}

#[tauri::command]
fn log_frontend_error(message: String) {
    app_log::warn(format!("frontend error: {}", message));
}

#[tauri::command]
fn log_frontend_event(message: String) {
    app_log::info(format!("frontend event: {}", message));
}

#[tauri::command]
fn get_usage_stats() -> StatsSnapshot {
    stats::load_stats_snapshot()
}

#[tauri::command]
fn get_local_data_status() -> Result<LocalDataStatus, String> {
    let loaded = config::load_config()?;
    let auto_hotword_status = hotword_history::status()?;
    Ok(LocalDataStatus {
        recent_context_enabled: loaded.data.context.enable_recent_context,
        recent_context_count: config::recent_context_count(),
        auto_hotwords_enabled: auto_hotword_status.enabled,
        auto_hotword_entry_count: auto_hotword_status.entry_count,
        auto_hotword_total_chars: auto_hotword_status.total_chars,
        stats_event_count: stats::stats_event_count(),
        screen_context_enabled: loaded.data.screen_context.enabled,
        llm_post_edit_enabled: loaded.data.llm_post_edit.enabled,
        restore_clipboard_after_paste: loaded.data.typing.restore_clipboard_after_paste,
    })
}

#[tauri::command]
fn clear_usage_stats() -> Result<ConnectionTestResult, String> {
    stats::clear_stats()?;
    app_log::info("用户清除使用统计数据。");
    Ok(ConnectionTestResult {
        message: "使用统计已清除。".to_string(),
    })
}

#[tauri::command]
fn clear_recent_context() -> Result<ConnectionTestResult, String> {
    config::clear_recent_context()?;
    app_log::info(format!(
        "用户清除最近上下文: remaining={}",
        config::recent_context_count()
    ));
    Ok(ConnectionTestResult {
        message: "最近上下文已清除。".to_string(),
    })
}

#[tauri::command]
fn get_auto_hotword_status() -> Result<hotword_history::AutoHotwordStatus, String> {
    hotword_history::status()
}

#[tauri::command]
fn clear_hotword_history() -> Result<ConnectionTestResult, String> {
    hotword_history::clear_history()?;
    let status = hotword_history::status().ok();
    app_log::info(format!(
        "用户清除自动热词采集文本: remaining_entries={}",
        status.map(|item| item.entry_count).unwrap_or(0)
    ));
    Ok(ConnectionTestResult {
        message: "自动热词采集文本已清空。".to_string(),
    })
}

#[tauri::command]
async fn generate_hotword_candidates(
    config: config::AppConfig,
) -> Result<hotword_generator::HotwordGenerationResult, String> {
    hotword_generator::generate_candidates(config).await
}

#[tauri::command]
async fn check_for_update() -> Result<update::UpdateStatus, String> {
    app_log::info("开始检查软件更新。");
    let loaded = config::load_config()?;
    update::check_for_update(&loaded.data.update)
        .await
        .map_err(|err| {
            app_log::warn(format!("软件更新检查失败: {}", err));
            err
        })
}

#[tauri::command]
async fn download_and_install_update(
    app: AppHandle,
) -> Result<update::InstallUpdateResult, String> {
    app_log::info("开始下载并安装软件更新。");
    let loaded = config::load_config()?;
    match update::download_and_install(&loaded.data.update).await {
        Ok(result) => {
            exit_after_update_installer_starts(app);
            Ok(result)
        }
        Err(err) => {
            app_log::warn(format!("下载并安装软件更新失败: {}", err));
            Err(err)
        }
    }
}

#[tauri::command]
fn get_overlay_text() -> overlay::OverlayText {
    overlay::OverlayText {
        text: overlay::current_text(),
    }
}

#[tauri::command]
fn list_audio_input_devices() -> Result<Vec<audio::AudioDeviceInfo>, String> {
    audio::list_input_devices()
}

#[tauri::command]
fn get_session_state(session: State<'_, SessionController>) -> session::SessionState {
    session.current_state()
}

#[tauri::command]
fn start_recording(
    app: AppHandle,
    session: State<'_, SessionController>,
) -> Result<session::SessionState, String> {
    session.start(Some(app))
}

#[tauri::command]
fn stop_recording(
    app: AppHandle,
    session: State<'_, SessionController>,
) -> Result<session::SessionState, String> {
    session.stop(Some(app))
}

#[tauri::command]
fn toggle_recording(
    app: AppHandle,
    session: State<'_, SessionController>,
) -> Result<session::SessionState, String> {
    session.toggle(Some(app))
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
            get_app_snapshot,
            get_setup_status,
            load_app_config,
            save_app_config,
            test_asr_config,
            test_llm_config,
            test_screen_context,
            open_setup_guide,
            open_log_file,
            get_diagnostic_report,
            copy_diagnostic_report_to_clipboard,
            copy_recent_input_text_to_clipboard,
            set_tray_language,
            hide_main_window,
            exit_application,
            update_close_preference,
            log_frontend_error,
            log_frontend_event,
            get_usage_stats,
            get_local_data_status,
            clear_usage_stats,
            clear_recent_context,
            get_auto_hotword_status,
            clear_hotword_history,
            generate_hotword_candidates,
            check_for_update,
            download_and_install_update,
            get_overlay_text,
            list_audio_input_devices,
            get_session_state,
            start_recording,
            stop_recording,
            toggle_recording
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

fn hotkey_equal(left: &str, right: &str) -> bool {
    let normalize = |value: &str| {
        value
            .split('+')
            .map(|part| part.trim().to_ascii_lowercase())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("+")
    };
    normalize(left) == normalize(right)
}

fn hotkey_registration_test_needed(previous: Option<&AppConfig>, next: &AppConfig) -> bool {
    if !next.triggers.hotkey_enabled {
        return false;
    }
    previous
        .map(|previous| {
            !previous.triggers.hotkey_enabled || !hotkey_equal(&previous.hotkey, &next.hotkey)
        })
        .unwrap_or(true)
}

fn hotkey_runtime_update_needed(previous: Option<&AppConfig>, next: &AppConfig) -> bool {
    previous
        .map(|previous| {
            previous.triggers.hotkey_enabled != next.triggers.hotkey_enabled
                || !hotkey_equal(&previous.hotkey, &next.hotkey)
        })
        .unwrap_or(true)
}

fn autostart_update_needed(previous: Option<&AppConfig>, next: &AppConfig) -> bool {
    previous
        .map(|previous| previous.startup.launch_on_startup != next.startup.launch_on_startup)
        .unwrap_or(true)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ConfigSideEffects {
    restart_hotkey: bool,
    apply_autostart: bool,
}

fn config_side_effects(previous: Option<&AppConfig>, next: &AppConfig) -> ConfigSideEffects {
    ConfigSideEffects {
        restart_hotkey: hotkey_runtime_update_needed(previous, next),
        apply_autostart: autostart_update_needed(previous, next),
    }
}

fn apply_config_side_effects(
    app: AppHandle,
    loaded: &LoadedConfig,
    side_effects: ConfigSideEffects,
) {
    hotkey::refresh_trigger_config_from(&loaded.data.triggers);
    overlay::update_config(&app, &loaded.data.ui);

    if side_effects.restart_hotkey {
        restart_hotkey_in_background(app);
    }

    if side_effects.apply_autostart {
        apply_autostart_in_background(loaded.data.startup.clone());
    }
}

fn restart_hotkey_in_background(app: AppHandle) {
    std::thread::spawn(move || {
        if let Err(err) = hotkey::restart_global_hotkey_thread(app) {
            app_log::warn(format!("配置已保存，但快捷键重新注册未确认完成: {}", err));
        }
    });
}

fn apply_autostart_in_background(startup: config::StartupConfig) {
    std::thread::spawn(move || {
        if let Err(err) = autostart::apply(&startup) {
            app_log::warn(format!("配置已保存，但开机自启动后台同步失败: {}", err));
        }
    });
}

fn exit_after_update_installer_starts(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(800));
        app_log::info("更新安装程序已启动，退出当前版本以释放安装文件。");
        app.exit(0);
    });
}

fn enabled_trigger_summary(config: &AppConfig) -> String {
    let mut triggers = Vec::new();
    if config.triggers.hotkey_enabled {
        triggers.push(config.hotkey.to_uppercase());
    }
    if config.triggers.right_alt_enabled {
        triggers.push("右 Alt".to_string());
    }
    if config.triggers.middle_mouse_enabled {
        triggers.push("鼠标中键".to_string());
    }
    if triggers.is_empty() {
        "未启用".to_string()
    } else {
        triggers.join(" / ")
    }
}

fn redact_user_path(value: &str) -> String {
    let Ok(profile) = std::env::var("USERPROFILE") else {
        return value.to_string();
    };
    redact_path_with_profile(value, &profile)
}

fn redact_path_with_profile(value: &str, profile: &str) -> String {
    if profile.is_empty() {
        return value.to_string();
    }
    let lower_value = value.to_ascii_lowercase();
    let lower_profile = profile.to_ascii_lowercase();
    if lower_value == lower_profile {
        return "%USERPROFILE%".to_string();
    }
    if lower_value.starts_with(&lower_profile) {
        let suffix = &value[profile.len()..];
        if suffix.starts_with('\\') || suffix.starts_with('/') {
            return format!("%USERPROFILE%{}", suffix);
        }
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        autostart_update_needed, blocking_validation_errors, build_setup_status,
        config_side_effects, enabled_trigger_summary, hotkey_registration_test_needed,
        hotkey_runtime_update_needed, redact_path_with_profile, AppConfig, ConfigSideEffects,
    };
    use crate::config::ConfigValidationError;

    #[test]
    fn hotkey_registration_test_is_needed_when_enabled_with_same_text() {
        let mut previous = AppConfig::default();
        previous.triggers.hotkey_enabled = false;

        let mut next = previous.clone();
        next.triggers.hotkey_enabled = true;

        assert!(hotkey_registration_test_needed(Some(&previous), &next));
    }

    #[test]
    fn hotkey_registration_test_is_skipped_when_still_disabled() {
        let mut previous = AppConfig::default();
        previous.triggers.hotkey_enabled = false;

        let next = previous.clone();

        assert!(!hotkey_registration_test_needed(Some(&previous), &next));
    }

    #[test]
    fn hotkey_runtime_update_is_skipped_for_unrelated_settings() {
        let previous = AppConfig::default();
        let mut next = previous.clone();
        next.typing.paste_delay_ms += 10;

        assert!(!hotkey_runtime_update_needed(Some(&previous), &next));
    }

    #[test]
    fn hotkey_runtime_update_is_needed_when_hotkey_changes() {
        let previous = AppConfig::default();
        let mut next = previous.clone();
        next.hotkey = "Ctrl + Space".to_string();

        assert!(hotkey_runtime_update_needed(Some(&previous), &next));
    }

    #[test]
    fn autostart_update_is_skipped_for_unrelated_settings() {
        let previous = AppConfig::default();
        let mut next = previous.clone();
        next.ui.width += 10;

        assert!(!autostart_update_needed(Some(&previous), &next));
    }

    #[test]
    fn autostart_update_is_needed_when_startup_changes() {
        let previous = AppConfig::default();
        let mut next = previous.clone();
        next.startup.launch_on_startup = !previous.startup.launch_on_startup;

        assert!(autostart_update_needed(Some(&previous), &next));
    }

    #[test]
    fn config_side_effects_are_empty_for_unrelated_settings() {
        let previous = AppConfig::default();
        let mut next = previous.clone();
        next.typing.paste_delay_ms += 10;

        assert_eq!(
            config_side_effects(Some(&previous), &next),
            ConfigSideEffects::default()
        );
    }

    #[test]
    fn config_side_effects_detect_hotkey_and_autostart_independently() {
        let previous = AppConfig::default();

        let mut hotkey_next = previous.clone();
        hotkey_next.hotkey = "Ctrl + Space".to_string();
        assert_eq!(
            config_side_effects(Some(&previous), &hotkey_next),
            ConfigSideEffects {
                restart_hotkey: true,
                apply_autostart: false,
            }
        );

        let mut autostart_next = previous.clone();
        autostart_next.startup.launch_on_startup = !previous.startup.launch_on_startup;
        assert_eq!(
            config_side_effects(Some(&previous), &autostart_next),
            ConfigSideEffects {
                restart_hotkey: false,
                apply_autostart: true,
            }
        );
    }

    #[test]
    fn unchanged_hidden_config_validation_errors_do_not_block_visible_saves() {
        let mut previous = AppConfig::default();
        previous.audio.sample_rate = 0;

        let mut next = previous.clone();
        next.ui.width += 10;

        let blocking = blocking_validation_errors(
            vec![validation_error("audio.sample_rate")],
            Some(&previous),
            &next,
        );

        assert!(blocking.is_empty());
    }

    #[test]
    fn changed_hidden_config_validation_errors_still_block_saves() {
        let previous = AppConfig::default();
        let mut next = previous.clone();
        next.audio.sample_rate = 0;

        let blocking = blocking_validation_errors(
            vec![validation_error("audio.sample_rate")],
            Some(&previous),
            &next,
        );

        assert_eq!(blocking.len(), 1);
        assert_eq!(blocking[0].field, "audio.sample_rate");
    }

    #[test]
    fn visible_config_validation_errors_still_block_even_when_unchanged() {
        let mut previous = AppConfig::default();
        previous.request.language = "auto".to_string();

        let next = previous.clone();

        let blocking = blocking_validation_errors(
            vec![validation_error("request.language")],
            Some(&previous),
            &next,
        );

        assert_eq!(blocking.len(), 1);
        assert_eq!(blocking[0].field, "request.language");
    }

    #[test]
    fn setup_status_blocks_missing_auth_audio_and_triggers() {
        let mut config = AppConfig::default();
        config.auth.app_key.clear();
        config.auth.access_key.clear();
        config.triggers.hotkey_enabled = false;
        config.triggers.middle_mouse_enabled = false;
        config.triggers.right_alt_enabled = false;

        let status = build_setup_status(config, false);
        let codes: Vec<&str> = status
            .warnings
            .iter()
            .map(|warning| warning.code.as_str())
            .collect();

        assert!(!status.ready);
        assert!(status.missing_auth);
        assert!(!status.has_audio_device);
        assert_eq!(status.warnings.len(), 3);
        assert!(codes.contains(&"ASR_AUTH_MISSING"));
        assert!(codes.contains(&"MIC_DEVICE_NOT_FOUND"));
        assert!(codes.contains(&"TRIGGER_DISABLED"));
        assert!(status
            .warnings
            .iter()
            .all(|warning| warning.level == "blocking"));
    }

    #[test]
    fn setup_status_is_ready_when_auth_audio_and_trigger_are_available() {
        let mut config = AppConfig::default();
        config.auth.app_key = "app-key".to_string();
        config.auth.access_key = "access-key".to_string();
        config.triggers.hotkey_enabled = true;
        config.triggers.middle_mouse_enabled = false;
        config.triggers.right_alt_enabled = false;
        config.context.enable_recent_context = true;

        let status = build_setup_status(config, true);

        assert!(status.ready);
        assert!(!status.missing_auth);
        assert!(status.has_audio_device);
        assert_eq!(status.hotkey, "ctrl+q");
        assert_eq!(status.paste_method, "ctrl_v");
        assert!(status.privacy_recent_context_enabled);
        assert!(status.warnings.is_empty());
    }

    #[test]
    fn setup_status_keeps_soft_options_non_blocking() {
        let mut config = AppConfig::default();
        config.auth.app_key = "app-key".to_string();
        config.auth.access_key = "access-key".to_string();
        config.triggers.hotkey_enabled = true;
        config.triggers.middle_mouse_enabled = true;
        config.triggers.right_alt_enabled = true;
        config.context.enable_recent_context = true;
        config.audio.mute_system_volume_while_recording = true;

        let status = build_setup_status(config, true);

        assert!(status.ready);
        assert!(status
            .warnings
            .iter()
            .all(|warning| warning.level != "blocking"));
    }

    #[test]
    fn enabled_trigger_summary_lists_active_triggers() {
        let mut config = AppConfig {
            hotkey: "ctrl+space".to_string(),
            ..Default::default()
        };
        config.triggers.hotkey_enabled = true;
        config.triggers.middle_mouse_enabled = true;
        config.triggers.right_alt_enabled = true;

        assert_eq!(
            enabled_trigger_summary(&config),
            "CTRL+SPACE / 右 Alt / 鼠标中键"
        );
    }

    #[test]
    fn enabled_trigger_summary_reports_no_active_trigger() {
        let mut config = AppConfig::default();
        config.triggers.hotkey_enabled = false;
        config.triggers.middle_mouse_enabled = false;
        config.triggers.right_alt_enabled = false;

        assert_eq!(enabled_trigger_summary(&config), "未启用");
    }

    #[test]
    fn redacts_exact_user_profile_path() {
        assert_eq!(
            redact_path_with_profile("C:\\Users\\Alice", "C:\\Users\\Alice"),
            "%USERPROFILE%"
        );
    }

    #[test]
    fn redacts_user_profile_child_path_case_insensitively() {
        assert_eq!(
            redact_path_with_profile(
                "C:\\Users\\Alice\\AppData\\Local\\VoxType\\config.toml",
                "c:\\users\\alice",
            ),
            "%USERPROFILE%\\AppData\\Local\\VoxType\\config.toml"
        );
    }

    #[test]
    fn does_not_redact_similar_user_profile_prefix() {
        assert_eq!(
            redact_path_with_profile("C:\\Users\\AliceBackup\\config.toml", "C:\\Users\\Alice"),
            "C:\\Users\\AliceBackup\\config.toml"
        );
    }

    fn validation_error(field: &str) -> ConfigValidationError {
        ConfigValidationError {
            field: field.to_string(),
            message: "invalid".to_string(),
        }
    }
}
