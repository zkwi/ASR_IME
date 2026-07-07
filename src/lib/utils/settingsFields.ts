// Maps validation fields to stable settings panels. Keep ids aligned with
// component root/panel ids so hidden advanced sections can open on errors.
export function settingsPanelForField(field: string) {
  if (field.startsWith("asr.")) return "settings-auth";
  if (field.startsWith("auth.")) return "settings-auth";
  if (field === "aliyun_asr.language_hint") return "settings-asr-language";
  if (field.startsWith("aliyun_asr.")) return "settings-auth";
  if (field === "request.language") return "settings-asr-language";
  if (field.startsWith("request.")) return "settings-asr-language";
  if (field === "llm_post_edit.use_recent_context") return "settings-prompt-context";
  if (
    field === "llm_post_edit.system_prompt" ||
    field === "llm_post_edit.user_prompt_template" ||
    field === "llm_post_edit.min_chars" ||
    field === "llm_post_edit.screen_context_max_chars" ||
    field === "llm_post_edit.screen_context_max_lines" ||
    field === "llm_post_edit.recent_context_max_chars" ||
    field === "llm_post_edit.reference_hotwords_limit"
  ) {
    return "settings-llm-prompt";
  }
  if (field.startsWith("llm_post_edit.")) return "settings-llm-api";
  if (field.startsWith("auto_hotwords.")) return "settings-auto-hotwords";
  if (field.startsWith("screen_context.")) return "settings-screen-context";
  if (field.startsWith("context.") && field !== "context.hotwords") return "settings-prompt-context";
  if (field.startsWith("context.")) return "settings-context";
  if (field === "typing.paste_method" || field === "typing.remove_trailing_period" || field === "typing.restore_clipboard_after_paste") return "settings-basic-output";
  if (field === "audio.input_device" || field === "audio.input_device_name") return "settings-audio";
  if (field.startsWith("audio.")) return "settings-recording-troubleshooting";
  if (field.startsWith("ui.")) return "settings-overlay";
  if (field.startsWith("startup.") || field === "tray.close_behavior") return "settings-window";
  if (field.startsWith("update.")) return "settings-update";
  if (field === "tray.show_startup_message" || field === "tray.startup_message_timeout_ms") return "settings-overlay";
  return "settings-output";
}
