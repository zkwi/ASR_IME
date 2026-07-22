export const userErrorCodes = [
  "CONFIG_MISSING",
  "ASR_AUTH_MISSING",
  "ASR_CONNECT_TIMEOUT",
  "ASR_CONNECT_FAILED",
  "ASR_NETWORK_FAILED",
  "ASR_FINAL_TIMEOUT",
  "ASR_CONNECTION_CLOSED",
  "EMPTY_TRANSCRIPT",
  "MIC_DEVICE_NOT_FOUND",
  "MIC_START_FAILED",
  "MIC_STREAM_FAILED",
  "CLIPBOARD_WRITE_FAILED",
  "PASTE_FAILED",
  "HOTKEY_REGISTER_FAILED",
  "SYSTEM_AUDIO_RESTORE_FAILED",
  "UPDATE_CHECK_FAILED",
  "UPDATE_DOWNLOAD_FAILED",
] as const;

export type UserErrorCode = (typeof userErrorCodes)[number];

export type UserErrorDetail = {
  title: string;
  cause: string;
  action: string;
};

export type UserErrorMap = Record<UserErrorCode, UserErrorDetail>;
export type RuntimeUserErrorMap = UserErrorMap & Record<string, UserErrorDetail | undefined>;
