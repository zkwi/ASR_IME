export type Section = "Home" | "Hotwords" | "ApiConfig" | "Options" | "Privacy" | "History";

export type AppSnapshot = {
  hotkey: string;
  current_version: string;
};

export type UsageStats = {
  session_count: number;
  total_seconds: number;
  total_chars: number;
  total_minutes_int: number;
  avg_chars_per_minute: number;
};

export type DailyUsageStats = {
  day: string;
  stats: UsageStats;
};

export type HistoryEvent = {
  created_at: string;
  duration_seconds: number;
  text_chars: number;
};

export type StatsSnapshot = {
  path: string;
  recent_24h: UsageStats;
  recent_7d: UsageStats;
  by_day: DailyUsageStats[];
  history: HistoryEvent[];
};

export type UpdateStatus = {
  current_version: string;
  latest_version: string;
  update_available: boolean;
  asset_name: string | null;
  asset_size: number | null;
  message: string;
};

export type InstallUpdateResult = {
  version: string;
  asset_name: string;
  message: string;
};

export type ConnectionTestResult = { message: string; elapsed_ms?: number; thinking_strategy?: string };
export type DiagnosticReport = { text: string };
export type ScreenContextTestResult = {
  available_languages: string[];
  selected_language: string | null;
  elapsed_ms: number;
  text: string;
  text_chars: number;
  image_width: number;
  image_height: number;
  warning: string | null;
};

export type ConfigValidationError = {
  field: string;
  message: string;
};

export type ConfigSaveError = {
  message: string;
  errors?: ConfigValidationError[];
};

export type AutoHotwordStatus = {
  enabled: boolean;
  entry_count: number;
  total_chars: number;
  max_history_chars: number;
};

export type LocalDataStatus = {
  recent_context_enabled: boolean;
  recent_context_count: number;
  auto_hotwords_enabled: boolean;
  auto_hotword_entry_count: number;
  auto_hotword_total_chars: number;
  stats_event_count: number;
  screen_context_enabled: boolean;
  llm_post_edit_enabled: boolean;
  restore_clipboard_after_paste: boolean;
};

export type HotwordCandidate = {
  word: string;
  category: string;
  reason: string;
  confidence: number;
  source_count: number;
};

export type SelectableHotwordCandidate = HotwordCandidate & { selected: boolean };

export type HotwordGenerationResult = {
  candidates: HotwordCandidate[];
  used_chars: number;
  warning: string | null;
};

export type PersistConfigOptions = {
  enforceAuth?: boolean;
  focusErrors?: boolean;
};

export type CloseToTrayRequest = {
  first_time: boolean;
  behavior: string;
};

export type HotkeyCaptureState = "idle" | "recording";

export type LoadedConfig = {
  path: string;
  exists: boolean;
  data: AppConfig;
};

export type SessionPhase =
  | "idle"
  | "starting"
  | "recording"
  | "stopping"
  | "waiting_final_result"
  | "post_editing"
  | "pasting"
  | "succeeded"
  | "failed";

export type SessionState = {
  recording: boolean;
  phase: SessionPhase;
  message: string;
  error_code: string | null;
};

export type AsrFinalText = {
  text: string;
  error: string | null;
  error_code: string | null;
  warning: string | null;
  warning_code: string | null;
};

export type LastSessionOutcome =
  | {
      kind: "success";
      text: string;
      warning: string | null;
      warningCode: string | null;
      createdAt: number;
    }
  | null;

export type UserErrorAction =
  | "retry_recording"
  | "open_api_config"
  | "open_options"
  | "open_setup_guide"
  | "copy_diagnostic_report"
  | "open_log";

export type AudioLevel = { level: number };
export type AudioDeviceInfo = { index: number; name: string; is_default: boolean };
export type OverlayMode = "single" | "double";
export type AsrConnectionStatus =
  | "missing_auth"
  | "configured_not_tested"
  | "testing"
  | "tested_ok"
  | "tested_failed";

export type TextContext = { text: string };

export type AppConfig = {
  hotkey: string;
  auth: { app_key: string; access_key: string; resource_id: string };
  audio: {
    sample_rate: number;
    channels: number;
    segment_ms: number;
    max_record_seconds: number;
    stop_grace_ms: number;
    silence_auto_stop_seconds: number;
    silence_level_threshold: number;
    mute_system_volume_while_recording: boolean;
    input_device: number | null;
  };
  request: {
    ws_url: string;
    model_name: string;
    language: string;
    enable_nonstream: boolean;
    enable_itn: boolean;
    enable_punc: boolean;
    enable_ddc: boolean;
    show_utterances: boolean;
    result_type: string;
    enable_accelerate_text: boolean | null;
    accelerate_score: number | null;
    end_window_size: number | null;
    force_to_speech_time: number | null;
    final_result_timeout_seconds: number;
  };
  context: {
    enable_recent_context: boolean;
    recent_context_rounds: number;
    hotwords: string[];
    prompt_context: TextContext[];
    recent_context: TextContext[];
  };
  screen_context: {
    enabled: boolean;
    capture_scope: string;
    max_chars: number;
    timeout_ms: number;
  };
  triggers: {
    hotkey_enabled: boolean;
    middle_mouse_enabled: boolean;
    right_alt_enabled: boolean;
  };
  typing: {
    paste_delay_ms: number;
    paste_method: string;
    remove_trailing_period: boolean;
    restore_clipboard_after_paste: boolean;
    clipboard_restore_delay_ms: number;
    clipboard_snapshot_max_bytes: number;
    clipboard_open_retry_count: number;
    clipboard_open_retry_interval_ms: number;
  };
  startup: { launch_on_startup: boolean };
  update: { auto_check_on_startup: boolean; github_repo: string };
  auto_hotwords: {
    enabled: boolean;
    accepted_hotwords: string[];
    max_history_chars: number;
    max_candidates: number;
    ignored_hotwords: string[];
  };
  llm_post_edit: {
    enabled: boolean;
    min_chars: number;
    use_recent_context: boolean;
    base_url: string;
    api_key: string;
    model: string;
    timeout_seconds: number;
    enable_thinking: boolean;
    thinking_strategy: string;
    system_prompt: string;
    user_prompt_template: string;
  };
  ui: {
    width: number;
    height: number;
    margin_bottom: number;
    opacity: number;
    scroll_interval_ms: number;
    background_color: string;
    text_color: string;
  };
  tray: {
    show_startup_message: boolean;
    startup_message_timeout_ms: number;
    close_behavior: string;
    close_to_tray_notice_shown: boolean;
  };
  debug: { print_transcript_to_console: boolean };
};

export type OverlayText = { text: string };
export type OverlayConfig = { ui: AppConfig["ui"] };

export type TriggerKey = keyof AppConfig["triggers"];
export type SoftConfigNoticeKey =
  | TriggerKey
  | "enable_recent_context";
