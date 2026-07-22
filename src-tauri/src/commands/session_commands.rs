use crate::{audio, config, overlay, session};
use session::SessionController;
use tauri::{AppHandle, State};

#[tauri::command]
pub(crate) fn get_overlay_text() -> overlay::OverlayText {
    overlay::current_payload()
}

#[tauri::command]
pub(crate) fn get_overlay_config() -> overlay::OverlayConfig {
    let ui = config::load_config()
        .map(|loaded| loaded.data.ui)
        .unwrap_or_else(|_| overlay::current_config());
    overlay::OverlayConfig { ui }
}

#[tauri::command]
pub(crate) fn list_audio_input_devices() -> Result<Vec<audio::AudioDeviceInfo>, String> {
    audio::list_input_devices()
}

#[tauri::command]
pub(crate) fn get_session_state(session: State<'_, SessionController>) -> session::SessionState {
    session.current_state()
}

#[tauri::command]
pub(crate) fn start_recording(
    app: AppHandle,
    session: State<'_, SessionController>,
) -> Result<session::SessionState, String> {
    session.start(Some(app))
}

#[tauri::command]
pub(crate) fn stop_recording(
    app: AppHandle,
    session: State<'_, SessionController>,
) -> Result<session::SessionState, String> {
    session.stop(Some(app))
}

#[tauri::command]
pub(crate) fn toggle_recording(
    app: AppHandle,
    session: State<'_, SessionController>,
) -> Result<session::SessionState, String> {
    session.toggle(Some(app))
}
