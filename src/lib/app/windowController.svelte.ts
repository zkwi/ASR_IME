import { getCurrentWindow } from "@tauri-apps/api/window";
import type {
  AppConfig,
  ConfigExitGuardRequest,
  CloseToTrayRequest,
  LoadedConfig,
} from "$lib/types/app";

type SafeInvoke = <T>(
  command: string,
  args?: Record<string, unknown>,
  quiet?: boolean,
) => Promise<T | null>;

type WindowControllerOptions = {
  getConfig: () => AppConfig;
  applyLoadedConfig: (loaded: LoadedConfig) => void;
  safeInvoke: SafeInvoke;
  retryFailedSave: () => Promise<LoadedConfig | null>;
  discardUnsavedChanges: () => void;
  onHidden: () => void;
};

export function createWindowController(options: WindowControllerOptions) {
  let closePromptVisible = $state(false);
  let closePromptFirstTime = $state(false);
  let closePromptBehavior = $state("close_to_tray");
  let saveFailurePromptVisible = $state(false);
  let pendingGuardAction = $state<ConfigExitGuardRequest["action"] | null>(null);

  function showClosePrompt(request: CloseToTrayRequest) {
    closePromptFirstTime = request.first_time;
    closePromptBehavior = request.behavior;
    closePromptVisible = true;
  }

  async function minimize() {
    try {
      await getCurrentWindow().minimize();
    } catch (error) {
      console.warn(error);
    }
  }

  async function toggleMaximize() {
    try {
      await getCurrentWindow().toggleMaximize();
    } catch (error) {
      console.warn(error);
    }
  }

  async function requestClose() {
    try {
      await getCurrentWindow().close();
    } catch (error) {
      console.warn(error);
    }
  }

  async function confirmClosePrompt() {
    await hideToTray(closePromptFirstTime && closePromptBehavior === "close_to_tray");
  }

  async function hideToTray(markSeen: boolean) {
    closePromptVisible = false;
    if (markSeen) {
      await saveClosePreference(options.getConfig().tray.close_behavior, true);
    }
    await options.safeInvoke<void>("hide_main_window", undefined, true);
    options.onHidden();
  }

  async function hideToTrayFromTitlebar() {
    await hideToTray(false);
  }

  async function closeWithoutFuturePrompt() {
    closePromptVisible = false;
    await saveClosePreference("close_to_tray", true);
    await options.safeInvoke<void>("hide_main_window", undefined, true);
  }

  async function exitFromPrompt() {
    closePromptVisible = false;
    await options.safeInvoke<void>("exit_application", undefined, true);
  }

  function showSaveFailurePrompt(request: ConfigExitGuardRequest) {
    closePromptVisible = false;
    pendingGuardAction = request.action;
    saveFailurePromptVisible = true;
  }

  async function retrySaveAndContinue() {
    const result = await options.retryFailedSave();
    if (!result) return;
    await continueGuardedAction();
  }

  async function discardAndContinue() {
    options.discardUnsavedChanges();
    await continueGuardedAction();
  }

  function cancelSaveFailurePrompt() {
    saveFailurePromptVisible = false;
    pendingGuardAction = null;
  }

  async function continueGuardedAction() {
    const action = pendingGuardAction;
    saveFailurePromptVisible = false;
    pendingGuardAction = null;
    await options.safeInvoke<void>("set_config_exit_guard", { active: false }, true);
    if (action === "exit") {
      await options.safeInvoke<void>("exit_application", undefined, true);
      return;
    }
    if (action === "window_close") await requestClose();
  }

  async function saveClosePreference(behavior: string, noticeShown: boolean) {
    const result = await options.safeInvoke<LoadedConfig>(
      "update_close_preference",
      {
        closeBehavior: behavior,
        closeToTrayNoticeShown: noticeShown,
      },
      true,
    );
    if (result) options.applyLoadedConfig(result);
  }

  return {
    get closePromptVisible() { return closePromptVisible; },
    get saveFailurePromptVisible() { return saveFailurePromptVisible; },
    showClosePrompt,
    showSaveFailurePrompt,
    minimize,
    toggleMaximize,
    requestClose,
    hideToTrayFromTitlebar,
    confirmClosePrompt,
    closeWithoutFuturePrompt,
    exitFromPrompt,
    retrySaveAndContinue,
    discardAndContinue,
    cancelSaveFailurePrompt,
  };
}
