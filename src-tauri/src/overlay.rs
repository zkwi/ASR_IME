use crate::config::{UiConfig, MIN_UI_HEIGHT};
use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use tauri::{
    utils::config::Color, AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Monitor,
    WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

const OVERLAY_LABEL: &str = "caption-overlay";
pub const STARTING_TEXT: &str = "正在启动麦克风...";
pub const RECORDING_TEXT: &str = "正在听你说话...";
pub const POST_EDITING_TEXT: &str = "正在润色...";
pub const EMPTY_TRANSCRIPT_TEXT: &str = "没有识别到文字，请重试一次。";
pub const PASTE_FAILED_TEXT: &str = "粘贴失败，文本已复制，可手动 Ctrl+V。";
const DEFAULT_TEXT: &str = RECORDING_TEXT;
const TRANSPARENT_BACKGROUND: Color = Color(0, 0, 0, 0);
static OVERLAY_TEXT: OnceLock<Mutex<OverlayText>> = OnceLock::new();
static OVERLAY_UI: OnceLock<Mutex<UiConfig>> = OnceLock::new();

#[derive(Debug, Clone, Serialize)]
pub struct OverlayText {
    pub text: String,
    pub status_code: Option<String>,
    pub fallback_text: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverlayConfig {
    pub ui: UiConfig,
}

pub fn create_overlay_window(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window(OVERLAY_LABEL).is_some() {
        return Ok(());
    }
    let window =
        WebviewWindowBuilder::new(app, OVERLAY_LABEL, WebviewUrl::App("/?overlay=1".into()))
            .title("ASR Caption")
            .inner_size(350.0, 64.0)
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .transparent(true)
            .background_color(TRANSPARENT_BACKGROUND)
            .focused(false)
            .visible(false)
            .build()
            .map_err(|err| format!("创建悬浮字幕窗失败: {}", err))?;

    if let Err(err) = window.set_background_color(Some(TRANSPARENT_BACKGROUND)) {
        crate::app_log::warn(format!("设置悬浮字幕窗透明背景失败: {}", err));
    }

    if let Err(err) = window.set_focusable(false) {
        crate::app_log::warn(format!("设置悬浮字幕窗不可聚焦失败: {}", err));
    }

    crate::app_log::info("悬浮字幕窗已创建");
    Ok(())
}

pub fn show_message(app: &AppHandle, ui: &UiConfig, text: impl Into<String>) {
    show_with_payload(app, ui, plain_payload(text.into()));
}

pub fn show_status(app: &AppHandle, ui: &UiConfig, code: &str, fallback_text: &str) {
    show_with_payload(app, ui, status_payload(code, fallback_text));
}

fn show_with_payload(app: &AppHandle, ui: &UiConfig, payload: OverlayText) {
    if let Err(err) = create_overlay_window(app) {
        crate::app_log::warn(err);
        return;
    }
    let Some(window) = app.get_webview_window(OVERLAY_LABEL) else {
        return;
    };
    let effective_height = effective_overlay_height(ui.height);
    let _ = window.set_size(LogicalSize::new(ui.width as f64, effective_height as f64));
    if let Some(monitor) = current_monitor(app).or_else(|| window.primary_monitor().ok().flatten())
    {
        let position = monitor.position();
        let size = monitor.size();
        let scale = monitor.scale_factor().max(1.0);
        let monitor_x = position.x as f64 / scale;
        let monitor_y = position.y as f64 / scale;
        let monitor_width = size.width as f64 / scale;
        let monitor_height = size.height as f64 / scale;
        let x = monitor_x + ((monitor_width - ui.width as f64).max(0.0) / 2.0);
        let y = monitor_y
            + (monitor_height - effective_height as f64 - ui.margin_bottom as f64).max(0.0);
        let _ = window.set_position(LogicalPosition::new(x, y));
    }
    update_config(app, ui);
    update_payload(app, payload);
    if let Err(err) = window.set_focusable(false) {
        crate::app_log::warn(format!("显示前设置悬浮字幕窗不可聚焦失败: {}", err));
    }

    if let Err(err) = window.show() {
        crate::app_log::warn(format!("显示悬浮字幕窗失败: {}", err));
    } else {
        let _ = window.set_focusable(false);
        refresh_visible_window(&window);
        crate::app_log::info("悬浮字幕窗已显示");
    }
}

fn effective_overlay_height(configured_height: u32) -> u32 {
    configured_height.max(MIN_UI_HEIGHT)
}

fn refresh_visible_window(window: &WebviewWindow) {
    if let Err(err) = window.set_always_on_top(true) {
        crate::app_log::warn(format!("刷新悬浮字幕置顶状态失败: {}", err));
    }

    platform_refresh_visible_window(window);
}

#[cfg(windows)]
fn platform_refresh_visible_window(window: &WebviewWindow) {
    use windows::Win32::Graphics::Gdi::{
        RedrawWindow, RDW_ALLCHILDREN, RDW_INVALIDATE, RDW_UPDATENOW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
    };

    let Ok(hwnd) = window.hwnd() else {
        return;
    };

    unsafe {
        if let Err(err) = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        ) {
            crate::app_log::warn(format!("刷新悬浮字幕窗口层级失败: {}", err));
        }

        // 长时间隐藏/显示后，透明 WebView 偶发会处于可见但不重绘的状态。
        let _ = RedrawWindow(
            Some(hwnd),
            None,
            None,
            RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN,
        );
    }
}

#[cfg(not(windows))]
fn platform_refresh_visible_window(_window: &WebviewWindow) {}

fn current_monitor(app: &AppHandle) -> Option<Monitor> {
    let cursor = cursor_position()?;
    let monitors = app.available_monitors().ok()?;
    monitors.into_iter().find(|monitor| {
        let position = monitor.position();
        let size = monitor.size();
        let left = position.x;
        let top = position.y;
        let right = left + size.width as i32;
        let bottom = top + size.height as i32;
        cursor.x >= left && cursor.x < right && cursor.y >= top && cursor.y < bottom
    })
}

fn cursor_position() -> Option<POINT> {
    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point).ok()? };
    Some(point)
}

pub fn update_text(app: &AppHandle, text: impl Into<String>) {
    update_payload(app, plain_payload(text.into()));
}

pub fn update_status(app: &AppHandle, code: &str, fallback_text: &str) {
    update_payload(app, status_payload(code, fallback_text));
}

fn update_payload(app: &AppHandle, payload: OverlayText) {
    set_current_text(payload.clone());
    let _ = app.emit_to(OVERLAY_LABEL, "overlay-text", payload);
}

pub fn update_config(app: &AppHandle, ui: &UiConfig) {
    set_current_config(ui);
    let _ = app.emit_to(
        OVERLAY_LABEL,
        "overlay-config",
        OverlayConfig { ui: ui.clone() },
    );
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
        let _ = window.hide();
    }
}

pub fn current_payload() -> OverlayText {
    OVERLAY_TEXT
        .get_or_init(|| Mutex::new(status_payload("recording", DEFAULT_TEXT)))
        .lock()
        .map(|text| text.clone())
        .unwrap_or_else(|_| status_payload("recording", DEFAULT_TEXT))
}

pub fn current_config() -> UiConfig {
    OVERLAY_UI
        .get_or_init(|| Mutex::new(UiConfig::default()))
        .lock()
        .map(|ui| ui.clone())
        .unwrap_or_default()
}

fn set_current_text(payload: OverlayText) {
    if let Ok(mut current) = OVERLAY_TEXT
        .get_or_init(|| Mutex::new(status_payload("recording", DEFAULT_TEXT)))
        .lock()
    {
        *current = if payload.text.trim().is_empty() {
            status_payload("recording", DEFAULT_TEXT)
        } else {
            payload
        };
    }
}

fn plain_payload(text: String) -> OverlayText {
    OverlayText {
        text,
        status_code: None,
        fallback_text: None,
    }
}

fn status_payload(code: &str, fallback_text: &str) -> OverlayText {
    OverlayText {
        text: fallback_text.to_string(),
        status_code: Some(code.to_string()),
        fallback_text: Some(fallback_text.to_string()),
    }
}

fn set_current_config(ui: &UiConfig) {
    if let Ok(mut current) = OVERLAY_UI
        .get_or_init(|| Mutex::new(UiConfig::default()))
        .lock()
    {
        *current = ui.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::{effective_overlay_height, status_payload};

    #[test]
    fn clamps_legacy_low_height_for_two_lines() {
        assert_eq!(effective_overlay_height(40), 52);
        assert_eq!(effective_overlay_height(51), 52);
        assert_eq!(effective_overlay_height(52), 52);
        assert_eq!(effective_overlay_height(88), 88);
    }

    #[test]
    fn status_payload_keeps_a_stable_code_and_fallback_text() {
        let payload = status_payload("starting", "正在启动麦克风...");

        assert_eq!(payload.status_code.as_deref(), Some("starting"));
        assert_eq!(payload.fallback_text.as_deref(), Some("正在启动麦克风..."));
        assert_eq!(payload.text, "正在启动麦克风...");
    }
}
