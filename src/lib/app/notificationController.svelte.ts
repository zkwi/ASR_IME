import type { CopyKey } from "$lib/i18n";

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
  let timer: number | undefined;

  function show(nextMessage: string, nextKind: ActionNoticeKind, nextAction: ActionNoticeAction | undefined = undefined) {
    message = nextMessage;
    kind = nextKind;
    action = nextAction ?? null;
    clearTimer();
    const baseDuration = nextAction ? 12_000 : nextKind === "error" ? 6400 : nextKind === "warning" ? 5200 : 3200;
    const extraDuration = nextMessage.length > 80 ? 1800 : 0;
    timer = window.setTimeout(() => {
      message = "";
      action = null;
      timer = undefined;
    }, baseDuration + extraDuration);
  }

  async function runAction() {
    const currentAction = action;
    if (!currentAction || currentAction.isBusy?.()) return;
    try {
      await currentAction.onClick();
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
