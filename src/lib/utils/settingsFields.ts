import type { Section } from "$lib/types/app";

export type AdvancedSection = Extract<Section, "Hotwords" | "ApiConfig" | "Options">;

export function fieldAdvancedSection(field: string): AdvancedSection {
  if (field.startsWith("context.") || field.startsWith("auto_hotwords.") || field === "llm_post_edit.system_prompt" || field === "llm_post_edit.user_prompt_template" || field === "llm_post_edit.min_chars") {
    return "Hotwords";
  }
  if (field.startsWith("request.") || field.startsWith("llm_post_edit.")) return "ApiConfig";
  return "Options";
}

export function fieldRequiresAdvancedSettings(field: string) {
  return (
    (field.startsWith("audio.") && field !== "audio.input_device") ||
    (field.startsWith("ui.") && field !== "ui.opacity") ||
    (field.startsWith("context.") && field !== "context.hotwords") ||
    field.startsWith("auto_hotwords.") ||
    (field.startsWith("request.") && field !== "request.final_result_timeout_seconds" && field !== "request.language") ||
    field.startsWith("update.") ||
    field === "typing.paste_delay_ms" ||
    field === "typing.clipboard_restore_delay_ms" ||
    field === "typing.clipboard_snapshot_max_bytes" ||
    field === "typing.clipboard_open_retry_count" ||
    field === "typing.clipboard_open_retry_interval_ms" ||
    field === "typing.restore_clipboard_after_paste" ||
    field === "llm_post_edit.enable_thinking"
  );
}

export function settingsPanelForField(field: string) {
  if (field.startsWith("auth.")) return "settings-auth";
  if (field === "request.language") return "settings-asr-language";
  if (field.startsWith("request.")) return "settings-request";
  if (field === "llm_post_edit.system_prompt" || field === "llm_post_edit.user_prompt_template" || field === "llm_post_edit.min_chars") {
    return "settings-llm-prompt";
  }
  if (field.startsWith("llm_post_edit.")) return "settings-llm-api";
  if (field.startsWith("auto_hotwords.")) return "settings-auto-hotwords";
  if (field.startsWith("context.") && field !== "context.hotwords") return "settings-prompt-context";
  if (field.startsWith("context.")) return "settings-context";
  if (field === "audio.input_device") return "settings-audio";
  if (field.startsWith("audio.")) return "settings-recording-troubleshooting";
  if (field.startsWith("ui.")) return "settings-overlay";
  if (field.startsWith("update.")) return "settings-update";
  if (field === "tray.show_startup_message" || field === "tray.startup_message_timeout_ms") return "settings-overlay";
  return "settings-output";
}
