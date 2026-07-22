use crate::app_log;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};

pub const MAIN_LABEL: &str = "main";
static CONFIG_EXIT_GUARD: AtomicBool = AtomicBool::new(false);

pub fn set_config_exit_guard(active: bool) {
    CONFIG_EXIT_GUARD.store(active, Ordering::SeqCst);
}

pub fn config_exit_guard_active() -> bool {
    CONFIG_EXIT_GUARD.load(Ordering::SeqCst)
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigExitGuardRequest {
    pub action: String,
}

pub fn request_config_exit_guard(app: &AppHandle, action: &str) -> bool {
    if !config_exit_guard_active() {
        return false;
    }
    show_existing(app, "配置保存失败时");
    let Some(window) = app.get_webview_window(MAIN_LABEL) else {
        return false;
    };
    if let Err(err) = window.emit(
        "config-exit-guard-requested",
        ConfigExitGuardRequest {
            action: action.to_string(),
        },
    ) {
        app_log::warn(format!("发送配置退出保护事件失败: {}", err));
        return false;
    }
    true
}

pub fn show_existing(app: &AppHandle, source: &str) {
    let Some(window) = app.get_webview_window(MAIN_LABEL) else {
        app_log::warn(format!("{}显示主窗口失败：找不到主窗口。", source));
        return;
    };
    if let Err(err) = window.unminimize() {
        app_log::warn(format!("{}恢复主窗口失败: {}", source, err));
    }
    if let Err(err) = window.show() {
        app_log::warn(format!("{}显示主窗口失败: {}", source, err));
    }
    if let Err(err) = window.set_focus() {
        app_log::warn(format!("{}聚焦主窗口失败: {}", source, err));
    }
}

#[cfg(test)]
mod tests {
    use super::{config_exit_guard_active, set_config_exit_guard};

    #[test]
    fn config_exit_guard_tracks_failed_unsaved_changes() {
        set_config_exit_guard(true);
        assert!(config_exit_guard_active());

        set_config_exit_guard(false);
        assert!(!config_exit_guard_active());
    }
}
