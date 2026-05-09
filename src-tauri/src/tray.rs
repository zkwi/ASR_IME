use crate::session::SessionController;
use crate::{app_log, config, main_window};
use std::sync::{Mutex, OnceLock};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{image::Image, AppHandle, Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

const TRAY_ID: &str = "voxtype";
const TRAY_ICON_SIZE: usize = 32;
const TRAY_ICON_RGBA: &[u8] = include_bytes!("../icons/32x32.rgba");

const OPEN_CONFIG_ID: &str = "open_config";
const OPEN_LOG_ID: &str = "open_log";
const OPEN_SETUP_GUIDE_ID: &str = "open_setup_guide";
const OPEN_ISSUES_ID: &str = "open_issues";
const CHECK_UPDATE_ID: &str = "check_update";
const RESTART_ID: &str = "restart";
const EXIT_ID: &str = "exit";
const CHECK_UPDATE_EVENT: &str = "check-update-requested";
const ISSUES_URL: &str = "https://github.com/zkwi/VoxType/issues/new/choose";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TrayLabels {
    open_config: &'static str,
    open_log: &'static str,
    open_setup_guide: &'static str,
    open_issues: &'static str,
    check_update: &'static str,
    restart: &'static str,
    exit: &'static str,
}

pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
    let language = current_tray_language();
    let menu = build_tray_menu(app, tray_labels(language))?;

    let app_for_event = app.clone();
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(tray_tooltip(language, false))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            OPEN_CONFIG_ID => open_config_file(app),
            OPEN_LOG_ID => {
                if let Err(err) = open_log_file(app) {
                    app_log::warn(err);
                }
            }
            OPEN_SETUP_GUIDE_ID => {
                if let Err(err) = crate::setup_guide::open(app) {
                    app_log::warn(err);
                }
            }
            OPEN_ISSUES_ID => {
                if let Err(err) = open_issues_page(app) {
                    app_log::warn(err);
                }
            }
            CHECK_UPDATE_ID => request_update_check(app),
            RESTART_ID => restart_app(app),
            EXIT_ID => exit_app(app),
            _ => {}
        })
        .on_tray_icon_event(move |_tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            }
            | TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => show_main_window(&app_for_event),
            _ => {}
        });
    let icon = normal_tray_icon();
    builder = builder.icon(icon);
    builder
        .build(app)
        .map_err(|err| format!("创建托盘图标失败: {}", err))?;
    Ok(())
}

pub fn set_language(app: &AppHandle, language: &str) -> Result<(), String> {
    let language = normalize_tray_language(language);
    if let Ok(mut current) = tray_language().lock() {
        *current = language;
    }

    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return Ok(());
    };
    let menu = build_tray_menu(app, tray_labels(language))?;
    tray.set_menu(Some(menu))
        .map_err(|err| format!("更新托盘菜单语言失败: {}", err))?;
    tray.set_tooltip(Some(tray_tooltip(language, current_tray_active())))
        .map_err(|err| format!("更新托盘提示语言失败: {}", err))?;
    Ok(())
}

fn build_tray_menu(app: &AppHandle, labels: TrayLabels) -> Result<Menu<tauri::Wry>, String> {
    let open_config =
        MenuItem::with_id(app, OPEN_CONFIG_ID, labels.open_config, true, None::<&str>)
            .map_err(|err| format!("创建托盘菜单失败: {}", err))?;
    let open_log = MenuItem::with_id(app, OPEN_LOG_ID, labels.open_log, true, None::<&str>)
        .map_err(|err| format!("创建托盘菜单失败: {}", err))?;
    let open_setup_guide = MenuItem::with_id(
        app,
        OPEN_SETUP_GUIDE_ID,
        labels.open_setup_guide,
        true,
        None::<&str>,
    )
    .map_err(|err| format!("创建托盘菜单失败: {}", err))?;
    let open_issues =
        MenuItem::with_id(app, OPEN_ISSUES_ID, labels.open_issues, true, None::<&str>)
            .map_err(|err| format!("创建托盘菜单失败: {}", err))?;
    let check_update = MenuItem::with_id(
        app,
        CHECK_UPDATE_ID,
        labels.check_update,
        true,
        None::<&str>,
    )
    .map_err(|err| format!("创建托盘菜单失败: {}", err))?;
    let restart = MenuItem::with_id(app, RESTART_ID, labels.restart, true, None::<&str>)
        .map_err(|err| format!("创建托盘菜单失败: {}", err))?;
    let separator = PredefinedMenuItem::separator(app)
        .map_err(|err| format!("创建托盘菜单分隔线失败: {}", err))?;
    let exit = MenuItem::with_id(app, EXIT_ID, labels.exit, true, None::<&str>)
        .map_err(|err| format!("创建托盘菜单失败: {}", err))?;
    Menu::with_items(
        app,
        &[
            &open_config,
            &open_log,
            &open_setup_guide,
            &open_issues,
            &check_update,
            &restart,
            &separator,
            &exit,
        ],
    )
    .map_err(|err| format!("创建托盘菜单失败: {}", err))
}

pub fn set_input_active(app: &AppHandle, active: bool) {
    if let Ok(mut current) = tray_active().lock() {
        *current = active;
    }
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let icon = if active {
        active_tray_icon()
    } else {
        normal_tray_icon()
    };
    if let Err(err) = tray.set_icon(Some(icon)) {
        app_log::warn(format!("更新托盘图标失败: {}", err));
    }
    let tooltip = tray_tooltip(current_tray_language(), active);
    if let Err(err) = tray.set_tooltip(Some(tooltip)) {
        app_log::warn(format!("更新托盘提示失败: {}", err));
    }
}

pub fn show_startup_message() {
    let Ok(loaded) = config::load_config() else {
        return;
    };
    if !loaded.data.tray.show_startup_message {
        return;
    }
    // Tauri v2 未内置 Windows 气泡通知；这里先写日志，后续可换成 notification 插件。
    app_log::info(format!(
        "声写已启动，按 {} / 右Alt / 鼠标中键 开始/停止语音输入",
        loaded.data.hotkey.to_uppercase()
    ));
}

fn tray_labels(language: &str) -> TrayLabels {
    match normalize_tray_language(language) {
        "en" => TrayLabels {
            open_config: "Open config file",
            open_log: "View logs",
            open_setup_guide: "Setup guide",
            open_issues: "Report issue",
            check_update: "Check updates",
            restart: "Restart app",
            exit: "Exit",
        },
        "zh-TW" => TrayLabels {
            open_config: "打開配置檔",
            open_log: "查看日誌",
            open_setup_guide: "配置指南",
            open_issues: "問題回報",
            check_update: "檢查更新",
            restart: "重新啟動程式",
            exit: "退出",
        },
        _ => TrayLabels {
            open_config: "打开配置文件",
            open_log: "查看日志",
            open_setup_guide: "配置指南",
            open_issues: "问题反馈",
            check_update: "检查更新",
            restart: "重启程序",
            exit: "退出",
        },
    }
}

fn tray_tooltip(language: &str, active: bool) -> &'static str {
    match (normalize_tray_language(language), active) {
        ("en", true) => "VoxType · Listening",
        ("en", false) => "VoxType",
        ("zh-TW", true) => "聲寫 · 輸入中",
        ("zh-TW", false) => "聲寫",
        (_, true) => "声写 · 输入中",
        (_, false) => "声写",
    }
}

fn normalize_tray_language(language: &str) -> &'static str {
    match language {
        "en" => "en",
        "zh-TW" => "zh-TW",
        _ => "zh-CN",
    }
}

fn current_tray_language() -> &'static str {
    tray_language()
        .lock()
        .map(|language| *language)
        .unwrap_or("zh-CN")
}

fn current_tray_active() -> bool {
    tray_active().lock().map(|active| *active).unwrap_or(false)
}

fn tray_language() -> &'static Mutex<&'static str> {
    static TRAY_LANGUAGE: OnceLock<Mutex<&'static str>> = OnceLock::new();
    TRAY_LANGUAGE.get_or_init(|| Mutex::new("zh-CN"))
}

fn tray_active() -> &'static Mutex<bool> {
    static TRAY_ACTIVE: OnceLock<Mutex<bool>> = OnceLock::new();
    TRAY_ACTIVE.get_or_init(|| Mutex::new(false))
}

fn open_config_file(app: &AppHandle) {
    match config::load_config() {
        Ok(loaded) => {
            let path = loaded.path.clone();
            if !loaded.exists {
                match config::save_config(loaded.data) {
                    Ok(created) => app_log::info(format!("已创建默认配置文件: {}", created.path)),
                    Err(err) => {
                        app_log::warn(format!("创建默认配置文件失败: {}", err));
                        return;
                    }
                }
            }
            if let Err(err) = app.opener().open_path(path, None::<&str>) {
                app_log::warn(format!("打开配置文件失败: {}", err));
            }
        }
        Err(err) => app_log::warn(format!("读取配置文件路径失败: {}", err)),
    }
}

pub fn open_log_file(app: &AppHandle) -> Result<(), String> {
    open_log_file_with_source(app, "托盘")
}

pub fn open_log_file_from_main(app: &AppHandle) -> Result<(), String> {
    open_log_file_with_source(app, "主窗口")
}

pub fn exit_app(app: &AppHandle) {
    crate::hotkey::stop_input_threads();
    let controller = app.state::<SessionController>().inner().clone();
    controller.abort_from_worker(app, "Application exiting.");
    app.exit(0);
}

fn restart_app(app: &AppHandle) {
    app_log::info("用户从托盘菜单重启程序。");
    crate::hotkey::stop_input_threads();
    let controller = app.state::<SessionController>().inner().clone();
    controller.abort_from_worker(app, "Application restarting.");
    app.request_restart();
}

fn open_issues_page(app: &AppHandle) -> Result<(), String> {
    app_log::info("用户从托盘菜单打开问题反馈页面。");
    app.opener()
        .open_url(ISSUES_URL, None::<&str>)
        .map_err(|err| format!("打开问题反馈页面失败: {}", err))
}

fn open_log_file_with_source(app: &AppHandle, source: &str) -> Result<(), String> {
    app_log::info(format!("用户从{}打开日志文件。", source));
    let path = app_log::log_path();
    app.opener()
        .open_path(path.to_string_lossy().to_string(), None::<&str>)
        .map_err(|err| format!("打开日志文件失败: {}", err))?;
    Ok(())
}

fn show_main_window(app: &AppHandle) {
    main_window::show_existing(app, "托盘菜单");
}

fn request_update_check(app: &AppHandle) {
    show_main_window(app);
    if let Err(err) = app.emit(CHECK_UPDATE_EVENT, ()) {
        app_log::warn(format!("发送托盘检查更新事件失败: {}", err));
    }
}

fn normal_tray_icon() -> Image<'static> {
    Image::new(TRAY_ICON_RGBA, TRAY_ICON_SIZE as u32, TRAY_ICON_SIZE as u32)
}

fn active_tray_icon() -> Image<'static> {
    let mut rgba = TRAY_ICON_RGBA.to_vec();
    paint_status_dot(&mut rgba);
    Image::new_owned(rgba, TRAY_ICON_SIZE as u32, TRAY_ICON_SIZE as u32)
}

fn paint_status_dot(rgba: &mut [u8]) {
    let center_x = 24_i32;
    let center_y = 24_i32;
    let border_radius_sq = 64_i32;
    let dot_radius_sq = 36_i32;
    for y in 0..TRAY_ICON_SIZE {
        for x in 0..TRAY_ICON_SIZE {
            let dx = x as i32 - center_x;
            let dy = y as i32 - center_y;
            let distance_sq = dx * dx + dy * dy;
            if distance_sq > border_radius_sq {
                continue;
            }
            let offset = (y * TRAY_ICON_SIZE + x) * 4;
            let color = if distance_sq <= dot_radius_sq {
                [34, 197, 94, 255]
            } else {
                [255, 255, 255, 255]
            };
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{tray_labels, tray_tooltip};

    #[test]
    fn tray_labels_follow_selected_language() {
        let labels = tray_labels("en");

        assert_eq!(labels.open_config, "Open config file");
        assert_eq!(labels.open_log, "View logs");
        assert_eq!(labels.open_setup_guide, "Setup guide");
        assert_eq!(labels.open_issues, "Report issue");
        assert_eq!(labels.check_update, "Check updates");
        assert_eq!(labels.restart, "Restart app");
        assert_eq!(labels.exit, "Exit");
    }

    #[test]
    fn tray_labels_support_traditional_chinese() {
        let labels = tray_labels("zh-TW");

        assert_eq!(labels.open_config, "打開配置檔");
        assert_eq!(labels.open_log, "查看日誌");
        assert_eq!(labels.open_setup_guide, "配置指南");
        assert_eq!(labels.open_issues, "問題回報");
        assert_eq!(labels.check_update, "檢查更新");
        assert_eq!(labels.restart, "重新啟動程式");
        assert_eq!(labels.exit, "退出");
    }

    #[test]
    fn tray_language_falls_back_to_simplified_chinese() {
        let labels = tray_labels("fr");

        assert_eq!(labels.open_config, "打开配置文件");
        assert_eq!(labels.open_issues, "问题反馈");
        assert_eq!(labels.restart, "重启程序");
        assert_eq!(tray_tooltip("fr", false), "声写");
        assert_eq!(tray_tooltip("en", true), "VoxType · Listening");
    }
}
