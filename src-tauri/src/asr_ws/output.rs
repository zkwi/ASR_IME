use crate::session::{SessionController, SessionPhase};
use crate::{app_log, config, hotword_history, overlay, stats, text_output};
use serde::Serialize;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub(super) const ATTENTION_OVERLAY_HOLD: Duration = Duration::from_millis(1_800);

#[derive(Debug, Clone, Serialize)]
pub struct AsrFinalText {
    pub text: String,
    pub error: Option<String>,
    pub error_code: Option<String>,
    pub warning: Option<String>,
    pub warning_code: Option<String>,
}

pub(super) fn record_successful_transcript_side_effects(
    app: &AppHandle,
    text: &str,
    duration: f64,
) {
    if let Err(err) = config::remember_recent_context(text) {
        app_log::warn(format!("写入 recent context 失败: {}", err));
    }
    if let Err(err) = hotword_history::append_transcript(text) {
        app_log::warn(format!("写入自动热词历史失败: {}", err));
    }
    if let Err(err) = stats::append_event(text, duration) {
        app_log::warn(err);
    } else if let Err(err) = app.emit("usage-stats-updated", stats::load_stats_snapshot()) {
        app_log::warn(format!("刷新统计事件发送失败: {}", err));
    }
}

pub(super) fn finish_output_sent_session(
    session: &SessionController,
    generation: u64,
    app: Option<&AppHandle>,
) -> bool {
    session
        .finish_generation(
            generation,
            app,
            SessionPhase::Succeeded,
            "Transcript output completed.",
            None,
        )
        .is_some()
}

pub(super) fn emit_successful_final_text(
    app: &AppHandle,
    text: &str,
    warning: Option<String>,
    warning_code: Option<String>,
) {
    let _ = app.emit(
        "asr-final-text",
        AsrFinalText {
            text: text.to_string(),
            error: None,
            error_code: None,
            warning,
            warning_code,
        },
    );
}

pub(super) fn should_hold_overlay_for_output_warning(
    warning: Option<&str>,
    warning_code: Option<&str>,
) -> bool {
    warning.is_some() && !text_output::is_quiet_output_warning_code(warning_code)
}

pub(super) fn handle_empty_transcript(
    app: &AppHandle,
    session: &SessionController,
    generation: u64,
) {
    app_log::info("ASR session finished: empty transcript");
    if session
        .finish_generation(
            generation,
            Some(app),
            SessionPhase::Failed,
            overlay::EMPTY_TRANSCRIPT_TEXT,
            Some("EMPTY_TRANSCRIPT"),
        )
        .is_none()
    {
        return;
    }
    if session.is_current_generation(generation) {
        overlay::update_text(app, overlay::EMPTY_TRANSCRIPT_TEXT);
    }
    let _ = app.emit(
        "asr-final-text",
        AsrFinalText {
            text: String::new(),
            error: Some(overlay::EMPTY_TRANSCRIPT_TEXT.to_string()),
            error_code: Some("EMPTY_TRANSCRIPT".to_string()),
            warning: None,
            warning_code: None,
        },
    );
    thread::sleep(ATTENTION_OVERLAY_HOLD);
    if session.is_current_generation(generation) {
        overlay::hide(app);
    }
}

pub(super) fn emit_error(
    app: &AppHandle,
    session: &SessionController,
    generation: u64,
    error_code: &str,
    error: String,
) {
    app_log::warn(&error);
    let message = if error_code == "PASTE_FAILED" {
        overlay::PASTE_FAILED_TEXT.to_string()
    } else {
        format!("识别失败: {}", error)
    };
    if session.is_current_generation(generation) {
        overlay::update_text(app, message);
    }
    let _ = app.emit(
        "asr-final-text",
        AsrFinalText {
            text: String::new(),
            error: Some(error),
            error_code: Some(error_code.to_string()),
            warning: None,
            warning_code: None,
        },
    );
    thread::sleep(ATTENTION_OVERLAY_HOLD);
    if session.is_current_generation(generation) {
        overlay::hide(app);
    }
}

#[cfg(test)]
mod tests {
    use super::{finish_output_sent_session, should_hold_overlay_for_output_warning};
    use crate::text_output::WARNING_CLIPBOARD_PARTIAL_RESTORE;

    #[test]
    fn holds_overlay_only_for_visible_output_warnings() {
        assert!(!should_hold_overlay_for_output_warning(None, None));
        assert!(!should_hold_overlay_for_output_warning(
            Some("quiet"),
            Some(WARNING_CLIPBOARD_PARTIAL_RESTORE),
        ));
        assert!(should_hold_overlay_for_output_warning(
            Some("visible"),
            Some("CLIPBOARD_RESTORE_FAILED"),
        ));
    }

    #[test]
    fn output_sent_finishes_pasting_before_clipboard_cleanup() {
        let session = crate::session::SessionController::default();
        assert!(session
            .set_phase_for_generation(
                0,
                None,
                crate::session::SessionPhase::Pasting,
                "Pasting.",
                None
            )
            .is_some());

        assert!(finish_output_sent_session(&session, 0, None));

        let state = session.current_state();
        assert!(!state.recording);
        assert_eq!(state.phase, crate::session::SessionPhase::Succeeded);
    }
}
