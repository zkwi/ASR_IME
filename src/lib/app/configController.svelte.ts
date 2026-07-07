import { invoke } from "@tauri-apps/api/core";
import type { CopyKey } from "$lib/i18n";
import type {
  AppConfig,
  ConfigSaveError,
  ConfigValidationError,
  LoadedConfig,
  PersistConfigOptions,
} from "$lib/types/app";
import { clonePlain, configFingerprint, validationErrorMap } from "$lib/utils/config";
import { formatFrontendError } from "$lib/utils/frontendDiagnostics";

type ConfigSaveState = "idle" | "pending" | "saving" | "saved";

type ConfigControllerOptions = {
  autoSaveDelayMs: number;
  t: (key: CopyKey, values?: Record<string, string>) => string;
  getConfig: () => AppConfig;
  setConfig: (config: AppConfig) => void;
  setSavedConfigFingerprint: (fingerprint: string) => void;
  getSettingsDirty: () => boolean;
  getConfigLoaded: () => boolean;
  setConfigLoaded: (value: boolean) => void;
  setConfigExists: (value: boolean) => void;
  setSnapshotHotkey: (hotkey: string) => void;
  getValidationErrors: () => Record<string, string>;
  setValidationErrors: (errors: Record<string, string>) => void;
  validateHotkey: (hotkey: string) => string;
  requireAuthFields: (showNotice: boolean, focusTarget: boolean) => boolean;
  syncSetupStatusFromConfig: (config: AppConfig) => void;
  scrollToSettingsPanel: (panelId: string) => void;
  focusFirstValidationError: (errors: ConfigValidationError[]) => void;
  setStatusMessage: (message: string) => void;
  hasTauriApi: () => boolean;
  canAutoSave: () => boolean;
  logFrontendError: (message: string) => void;
  onConfigSaved?: (loaded: LoadedConfig) => void;
};

export function createConfigController(options: ConfigControllerOptions) {
  let saving = $state(false);
  let configSavedRecently = $state(false);
  let autoSaveTimer: number | undefined;
  let configSavedIndicatorTimer: number | undefined;

  function clearAutoSaveTimer() {
    if (autoSaveTimer !== undefined) window.clearTimeout(autoSaveTimer);
    autoSaveTimer = undefined;
  }

  function clearConfigSavedIndicatorTimer() {
    if (configSavedIndicatorTimer !== undefined) window.clearTimeout(configSavedIndicatorTimer);
    configSavedIndicatorTimer = undefined;
  }

  function markConfigSavedRecently() {
    clearConfigSavedIndicatorTimer();
    configSavedRecently = true;
    configSavedIndicatorTimer = window.setTimeout(() => {
      configSavedRecently = false;
      configSavedIndicatorTimer = undefined;
    }, 2400);
  }

  function applyLoadedConfig(loaded: LoadedConfig) {
    options.setConfig(loaded.data);
    options.setSavedConfigFingerprint(configFingerprint(loaded.data));
    options.setConfigExists(loaded.exists);
  }

  function scheduleAutoSaveConfig() {
    if (!options.canAutoSave()) return;
    clearAutoSaveTimer();
    autoSaveTimer = window.setTimeout(() => {
      autoSaveTimer = undefined;
      void autoSaveConfig();
    }, options.autoSaveDelayMs);
  }

  async function autoSaveConfig() {
    if (!options.canAutoSave() || !options.getSettingsDirty()) return;
    if (saving) {
      scheduleAutoSaveConfig();
      return;
    }
    await persistConfig({ enforceAuth: false, focusErrors: false });
  }

  async function persistConfig(saveOptions: PersistConfigOptions = {}) {
    const { enforceAuth = true, focusErrors = true } = saveOptions;
    if (saving) return null;
    const configToSave = clonePlain(options.getConfig());
    const saveFingerprint = configFingerprint(configToSave);
    saving = true;
    try {
      options.setValidationErrors({});
      const hotkeyError = options.validateHotkey(configToSave.hotkey);
      if (hotkeyError) {
        options.setValidationErrors({ hotkey: hotkeyError });
        options.setStatusMessage(hotkeyError);
        if (focusErrors) options.scrollToSettingsPanel("settings-output");
        return null;
      }
      if (enforceAuth && !options.requireAuthFields(focusErrors, focusErrors)) return null;
      if (!options.hasTauriApi()) {
        options.setStatusMessage(options.t("browserPreview"));
        return null;
      }
      const result = await invoke<LoadedConfig>("save_app_config", { config: configToSave });
      if (result) {
        const resultFingerprint = configFingerprint(result.data);
        const currentFingerprint = configFingerprint(options.getConfig());
        options.setSavedConfigFingerprint(resultFingerprint);
        if (currentFingerprint === saveFingerprint) options.setConfig(result.data);
        options.setSnapshotHotkey(result.data.hotkey);
        options.syncSetupStatusFromConfig(result.data);
        options.setConfigExists(result.exists);
        options.setConfigLoaded(true);
        options.setStatusMessage(options.t("configSaved"));
        markConfigSavedRecently();
        options.onConfigSaved?.(result);
      }
      return result;
    } catch (error) {
      const saveError = parseConfigSaveError(error);
      const errors = saveError.errors ?? [];
      options.setValidationErrors(validationErrorMap(errors));
      if (focusErrors) options.focusFirstValidationError(errors);
      options.setStatusMessage(saveError.message || options.t("validationFailed"));
      options.logFrontendError(`save config failed: ${formatFrontendError(error)}`);
      return null;
    } finally {
      saving = false;
    }
  }

  function parseConfigSaveError(error: unknown): ConfigSaveError {
    if (typeof error === "object" && error !== null) {
      const maybeError = error as { message?: unknown; errors?: unknown };
      return {
        message: typeof maybeError.message === "string" ? maybeError.message : options.t("validationFailed"),
        errors: Array.isArray(maybeError.errors) ? (maybeError.errors as ConfigValidationError[]) : [],
      };
    }
    return {
      message: typeof error === "string" ? error : options.t("validationFailed"),
      errors: [],
    };
  }

  function fieldError(field: string) {
    return options.getValidationErrors()[field] ?? "";
  }

  function configSaveState(isOverlay: boolean, isToast: boolean): ConfigSaveState {
    if (!options.getConfigLoaded() || !options.hasTauriApi() || isOverlay || isToast) return "idle";
    if (saving) return "saving";
    if (options.getSettingsDirty()) return "pending";
    if (configSavedRecently) return "saved";
    return "idle";
  }

  function dispose() {
    clearAutoSaveTimer();
    clearConfigSavedIndicatorTimer();
  }

  return {
    get saving() { return saving; },
    get configSavedRecently() { return configSavedRecently; },
    applyLoadedConfig,
    scheduleAutoSaveConfig,
    clearAutoSaveTimer,
    persistConfig,
    fieldError,
    configSaveState,
    dispose,
  };
}
