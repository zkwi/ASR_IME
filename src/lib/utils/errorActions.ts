import type { UserErrorAction } from "$lib/types/app";

export function actionsForUserError(code?: string | null): UserErrorAction[] {
  switch (code) {
    case "CONFIG_MISSING":
    case "ASR_AUTH_MISSING":
      return ["open_api_config", "open_setup_guide"];
    case "ASR_NETWORK_FAILED":
    case "ASR_FINAL_TIMEOUT":
      return ["retry_recording", "copy_diagnostic_report"];
    case "EMPTY_TRANSCRIPT":
      return ["retry_recording"];
    case "MIC_DEVICE_NOT_FOUND":
    case "MIC_START_FAILED":
    case "MIC_STREAM_FAILED":
      return ["open_options", "copy_diagnostic_report"];
    case "CLIPBOARD_WRITE_FAILED":
    case "PASTE_FAILED":
      return ["copy_diagnostic_report", "open_log"];
    case "HOTKEY_REGISTER_FAILED":
    case "SYSTEM_AUDIO_RESTORE_FAILED":
      return ["open_options", "copy_diagnostic_report"];
    default:
      return ["retry_recording", "copy_diagnostic_report"];
  }
}
