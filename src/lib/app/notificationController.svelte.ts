import type { CopyKey } from "$lib/i18n";
import {
  ERROR_NOTICE_RETENTION_ROUNDS,
  noticeAutoDismissMs,
} from "$lib/utils/notificationPolicy";

export type ActionNoticeKind = "success" | "info" | "warning" | "error";

export type ActionNoticeAction = {
  label: string;
  busyLabel?: string;
  isBusy?: () => boolean;
  onClick: () => void | Promise<void>;
};

type NotificationControllerOptions = {
  t: (key: CopyKey, values?: Record<string, string>) => string;
  setStatusMessage: (message: string) => void;
  logError: (message: string) => void;
};

export function createNotificationController(options: NotificationControllerOptions) {
  let message = $state("");
  let kind = $state<ActionNoticeKind>("success");
  let action = $state<ActionNoticeAction | null>(null);
  let remainingErrorRounds = 0;
  let timer: number | undefined;

  function show(nextMessage: string, nextKind: ActionNoticeKind, nextAction: ActionNoticeAction | undefined = undefined) {
    message = nextMessage;
    kind = nextKind;
    action = nextAction ?? null;
    remainingErrorRounds = nextKind === "error" ? ERROR_NOTICE_RETENTION_ROUNDS : 0;
    clearTimer();
    const duration = noticeAutoDismissMs(nextKind, Boolean(nextAction), nextMessage.length);
    if (duration !== null) {
      timer = window.setTimeout(() => {
        clear();
      }, duration);
    }
  }

  async function runAction() {
    const currentAction = action;
    if (!currentAction || currentAction.isBusy?.()) return;
    try {
      await currentAction.onClick();
      if (action === currentAction) clear();
    } catch (error) {
      options.logError(`action notice handler failed: ${formatError(error)}`);
      const failureMessage = options.t("operationFailedGeneric");
      options.setStatusMessage(failureMessage);
      show(failureMessage, "error");
    }
  }

  function clearTimer() {
    if (timer !== undefined) window.clearTimeout(timer);
    timer = undefined;
  }

  function clear() {
    clearTimer();
    message = "";
    action = null;
    remainingErrorRounds = 0;
  }

  function advanceSessionRound() {
    if (!message || kind !== "error") return;
    remainingErrorRounds = Math.max(0, remainingErrorRounds - 1);
    if (remainingErrorRounds === 0) clear();
  }

  function dispose() {
    clearTimer();
  }

  return {
    get message() { return message; },
    get kind() { return kind; },
    get actionLabel() { return action?.label ?? ""; },
    get actionBusyLabel() { return action?.busyLabel ?? ""; },
    get actionBusy() { return action?.isBusy?.() ?? false; },
    show,
    runAction,
    advanceSessionRound,
    clear,
    dispose,
  };
}

function formatError(error: unknown) {
  if (error instanceof Error) return error.stack || error.message;
  if (typeof error === "string") return error;
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}
