export type NoticeKind = "success" | "info" | "warning" | "error";

export const ERROR_NOTICE_RETENTION_ROUNDS = 5;

export function noticeAutoDismissMs(kind: NoticeKind, hasAction: boolean, messageLength: number) {
  if (kind === "error" || hasAction) return null;
  const baseDuration = kind === "warning" ? 8000 : 3200;
  return baseDuration + (messageLength > 80 ? 1800 : 0);
}

export function noticeRole(kind: NoticeKind): "alert" | "status" {
  return kind === "error" ? "alert" : "status";
}
