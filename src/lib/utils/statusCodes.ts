import type { CopyKey, UserErrorCode } from "$lib/i18n";

type Translate = (key: CopyKey) => string;

const overlayStatusKeys: Record<string, CopyKey | undefined> = {
  starting: "overlayStatusStarting",
  recording: "overlayStatusRecording",
  post_editing: "overlayStatusPostEditing",
  empty: "overlayStatusEmpty",
  paste_failed: "overlayStatusPasteFailed",
};

export function overlayStatusText(
  statusCode: string | null | undefined,
  fallbackText: string,
  t: Translate,
) {
  const key = statusCode ? overlayStatusKeys[statusCode] : undefined;
  return key ? t(key) : fallbackText;
}

export function invokeErrorCode(command: string): UserErrorCode | null {
  if (command === "check_for_update") return "UPDATE_CHECK_FAILED";
  if (command === "download_and_install_update") return "UPDATE_DOWNLOAD_FAILED";
  return null;
}
