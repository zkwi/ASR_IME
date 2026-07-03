pub(crate) mod config_commands;
pub(crate) mod diagnostic_commands;
pub(crate) mod session_commands;
pub(crate) mod update_commands;

use crate::config;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct AppSnapshot {
    pub(crate) hotkey: String,
    pub(crate) current_version: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct SetupStatus {
    pub(crate) ready: bool,
    pub(crate) missing_auth: bool,
    pub(crate) has_audio_device: bool,
    pub(crate) hotkey: String,
    pub(crate) paste_method: String,
    pub(crate) privacy_recent_context_enabled: bool,
    pub(crate) warnings: Vec<SetupWarning>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SetupWarning {
    pub(crate) code: String,
    pub(crate) level: String,
    pub(crate) title: String,
    pub(crate) message: String,
    pub(crate) action: String,
}

#[derive(Serialize)]
pub(crate) struct ConnectionTestResult {
    pub(crate) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) elapsed_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) thinking_strategy: Option<String>,
}

impl ConnectionTestResult {
    pub(crate) fn message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            elapsed_ms: None,
            thinking_strategy: None,
        }
    }

    pub(crate) fn with_llm_result(
        message: impl Into<String>,
        elapsed_ms: u64,
        thinking_strategy: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            elapsed_ms: Some(elapsed_ms),
            thinking_strategy: Some(thinking_strategy.into()),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct LocalDataStatus {
    pub(crate) config_path: String,
    pub(crate) log_path: String,
    pub(crate) recent_context_enabled: bool,
    pub(crate) recent_context_count: usize,
    pub(crate) auto_hotwords_enabled: bool,
    pub(crate) auto_hotword_entry_count: usize,
    pub(crate) auto_hotword_total_chars: usize,
    pub(crate) stats_event_count: usize,
    pub(crate) screen_context_enabled: bool,
    pub(crate) llm_post_edit_enabled: bool,
    pub(crate) restore_clipboard_after_paste: bool,
}

#[derive(Serialize)]
pub(crate) struct ConfigSaveError {
    pub(crate) message: String,
    pub(crate) errors: Vec<config::ConfigValidationError>,
}

#[derive(Serialize)]
pub(crate) struct DiagnosticReport {
    pub(crate) text: String,
}
