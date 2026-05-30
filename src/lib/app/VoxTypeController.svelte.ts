import { onMount } from "svelte";
import { browser } from "$app/environment";
import { createAutoHotwordsController } from "$lib/app/autoHotwordsController.svelte";
import { createDiagnosticsController } from "$lib/app/diagnosticsController.svelte";
import { createHotkeyCaptureController } from "$lib/app/hotkeyCaptureController.svelte";
import { createNotificationController } from "$lib/app/notificationController.svelte";
import { createOverlayController } from "$lib/app/overlayController.svelte";
import { createPrivacyController } from "$lib/app/privacyController.svelte";
import { createSettingsNavigationController } from "$lib/app/settingsNavigationController.svelte";
import { createStatsController } from "$lib/app/statsController.svelte";
import { createUpdateController } from "$lib/app/updateController.svelte";
import { createWindowController } from "$lib/app/windowController.svelte";
import type { SetupStatusItem } from "$lib/components/overview/SetupStatusCard.svelte";
import {
  autoSaveDelayMs,
  chineseTypingCharsPerMinute,
  fallbackConfig,
  fallbackSnapshot,
  micBars,
  overlayColorPresets,
  overlayOpacityPresets,
  setupStatusCacheKey,
} from "$lib/app/defaults";
import { copy, type CopyKey, type Language, type UserErrorDetail } from "$lib/i18n";
import { buildFinalPromptPreview } from "$lib/utils/autoHotwords";
import {
  configSetupMessage as getConfigSetupMessage,
  hasAuth as configHasAuth,
  hasLlmApiConfig as configHasLlmApiConfig,
  isConfigError,
  isErrorStatus as isUserErrorStatus,
  requiresAsrAuth as configRequiresAsrAuth,
  settingsPanelForError,
  shouldOpenSettingsForError,
  userErrorDetail as getUserErrorDetail,
  userErrorMessage as getUserErrorMessage,
} from "$lib/utils/appRouting";
import { actionsForUserError } from "$lib/utils/errorActions";
import { clonePlain, configFingerprint, validationErrorMap } from "$lib/utils/config";
import {
  clampAudioLevel,
  micBarHeight as getMicBarHeight,
  micBarOpacity as getMicBarOpacity,
} from "$lib/utils/audioMeter";
import {
  candidateConfidenceLabel,
  dedupeHotwords,
  effectiveHotwords as mergedEffectiveHotwords,
  hotwordCount as countManualHotwords,
  normalizeHotwords,
} from "$lib/utils/hotwords";
import {
  formatFrontendError,
  frontendMode as getFrontendMode,
  hasTauriApi,
  logFrontendError,
  logFrontendEvent,
} from "$lib/utils/frontendDiagnostics";
import { formatHotkey } from "$lib/utils/hotkeys";
import { overlayOpacityLabel } from "$lib/utils/overlayAppearance";
import {
  isBlockingSessionPhase,
  isQuietAsrWarningCode,
  sessionPhaseMessageKey,
} from "$lib/utils/sessionState";
import { userFacingInvokeFailure } from "$lib/utils/userFacingErrors";
import {
  formatHours,
  formatNumber as formatNumberForLanguage,
  formatSavedHours as formatSavedHoursForLanguage,
} from "$lib/utils/stats";
import {
  asrConfigFingerprint as buildAsrConfigFingerprint,
  asrConnectionStatusOk as isAsrConnectionStatusOk,
  asrConnectionStatusText as getAsrConnectionStatusText,
  buildLocalSetupStatus,
  buildSetupStatusItems,
  currentAsrConnectionStatus as getCurrentAsrConnectionStatus,
  formatEnabledTriggers as getEnabledTriggersText,
  localizeSetupWarnings,
  mergeSetupStatusFromConfig,
  pasteMethodLabel as getPasteMethodLabel,
  readCachedSetupStatus,
  setupActionText as getSetupActionText,
  type SetupStatus,
} from "$lib/utils/setupStatus";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AppConfig,
  AppSnapshot,
  AsrConnectionStatus,
  AsrFinalText,
  AudioDeviceInfo,
  AudioLevel,
  CloseToTrayRequest,
  ConfigSaveError,
  ConfigValidationError,
  ConnectionTestResult,
  LastSessionOutcome,
  LoadedConfig,
  LocalDataStatus,
  OverlayConfig,
  OverlayText,
  PersistConfigOptions,
  ScreenContextTestResult,
  SelectableHotwordCandidate,
  SessionPhase,
  SessionState,
  SoftConfigNoticeKey,
  StatsSnapshot,
  UserErrorAction,
} from "$lib/types/app";

const copyRecentInputCommand = "copy_recent_input_text_to_clipboard";
type ConfigSaveState = "idle" | "pending" | "saving" | "saved";

export function createVoxTypeController() {

  let snapshot = $state<AppSnapshot>(fallbackSnapshot);
  let config = $state<AppConfig>(clonePlain(fallbackConfig));
  let savedConfigFingerprint = $state(configFingerprint(fallbackConfig));
  let settingsDirty = $derived(configFingerprint(config) !== savedConfigFingerprint);
  let recording = $state(false);
  let sessionPhase = $state<SessionPhase>("idle");
  let sessionErrorCode = $state<string | null>(null);
  let lastSessionOutcome = $state<LastSessionOutcome>(null);
  let language = $state<Language>("zh-CN");
  let statusMessage = $state(copy["zh-CN"].bridgeLoading);
  let promptPreviewText = $state("");
  let saving = $state(false);
  let configSavedRecently = $state(false);
  let configExists = $state(true);
  let configLoaded = $state(false);
  let audioLevel = $state(0);
  const initialParams = browser ? new URLSearchParams(window.location.search) : new URLSearchParams();
  let audioDevices = $state<AudioDeviceInfo[]>([]);
  let isOverlay = $state(initialParams.has("overlay"));
  let isToast = $state(initialParams.has("toast"));
  let toastHotkey = $state(initialParams.get("hotkey") || "Ctrl + Q");
  const notifications = createNotificationController({
    t,
    setStatusMessage: (message) => {
      statusMessage = message;
    },
    logError: logFrontendError,
  });
  const stats = createStatsController({
    t,
    getLanguage: () => language,
  });
  const overlay = createOverlayController({
    getConfig: () => config,
    updateUi: (ui) => {
      config.ui = { ...config.ui, ...ui };
    },
    isOverlay: () => isOverlay,
    isRecording: () => recording,
    getAudioLevel: () => audioLevel,
    safeInvoke,
  });
  let uiCompact = $state(false);
  let setupStatus = $state<SetupStatus | null>(readCachedSetupStatus(browser));
  let testingAsr = $state(false);
  let asrConnectionStatus = $state<AsrConnectionStatus>("missing_auth");
  let asrTestedConfigFingerprint = $state("");
  let testingLlm = $state(false);
  let testingScreenContext = $state(false);
  let screenContextTestResult = $state<ScreenContextTestResult | null>(null);
  let validationErrors = $state<Record<string, string>>({});
  const autoHotwords = createAutoHotwordsController({
    getConfig: () => config,
    t,
    fieldError,
    effectiveHotwords,
    getStatusMessage: () => statusMessage,
    setStatusMessage: (message) => {
      statusMessage = message;
    },
    showActionNotice: notifications.show,
    safeInvoke,
    canConfirm: () => browser,
  });
  const privacy = createPrivacyController({
    t,
    safeInvoke,
    showActionNotice: notifications.show,
    canConfirm: () => browser,
    refreshStats,
    refreshAutoHotwordStatus: autoHotwords.refreshStatus,
  });
  const updates = createUpdateController({
    t,
    safeInvoke,
    canAutoCheck: () => !isOverlay && !isToast && configExists && config.update.auto_check_on_startup,
    currentVersion: () => snapshot.current_version,
    getStatusMessage: () => statusMessage,
    setStatusMessage: (message) => {
      statusMessage = message;
    },
    showActionNotice: notifications.show,
  });
  const diagnostics = createDiagnosticsController({
    hasTauriApi,
    t,
    setStatusMessage: (message) => {
      statusMessage = message;
    },
    showActionNotice: notifications.show,
    logError: logFrontendError,
  });
  const windows = createWindowController({
    getConfig: () => config,
    applyLoadedConfig,
    safeInvoke,
  });
  let succeededIdleTimer: number | undefined;
  let autoSaveTimer: number | undefined;
  let configSavedIndicatorTimer: number | undefined;
  let setupStatusLoading = $state(false);
  const hotkeyCapture = createHotkeyCaptureController({
    getConfig: () => config,
    t,
    getValidationErrors: () => validationErrors,
    setValidationErrors: (errors) => {
      validationErrors = errors;
    },
  });
  const settingsNav = createSettingsNavigationController({
    isBrowser: () => browser,
    requiresAsrAuth,
  });
  onMount(() => {
    const onError = (event: ErrorEvent) => {
      logFrontendError(`${event.message} (${event.filename}:${event.lineno}:${event.colno})`);
    };
    const onUnhandledRejection = (event: PromiseRejectionEvent) => {
      const reason = event.reason instanceof Error ? event.reason.stack || event.reason.message : String(event.reason);
      logFrontendError(`unhandled rejection: ${reason}`);
    };
    window.addEventListener("error", onError);
    window.addEventListener("unhandledrejection", onUnhandledRejection);
    document.getElementById("boot-fallback")?.remove();
    const params = new URLSearchParams(window.location.search);
    isOverlay = params.has("overlay");
    isToast = params.has("toast");
    toastHotkey = params.get("hotkey") || toastHotkey;
    refreshMainDensity();
    window.addEventListener("resize", refreshMainDensity);
    logFrontendEvent(
      `mounted mode=${frontendMode()} viewport=${window.innerWidth}x${window.innerHeight} dpr=${window.devicePixelRatio.toFixed(2)} compact=${uiCompact} language=${navigator.language} userAgent=${navigator.userAgent}`,
    );
    const savedLanguage = localStorage.getItem("voxtype-language");
    if (savedLanguage === "zh-CN" || savedLanguage === "zh-TW" || savedLanguage === "en") {
      language = savedLanguage;
      statusMessage = t("bridgeLoading");
      syncTrayLanguage(savedLanguage);
    }
    void bootstrapApp();
    let overlayPoll: number | undefined;
    if (isOverlay) {
      overlay.applyText("", true);
      window.addEventListener("resize", overlay.refreshLayout);
      overlayPoll = window.setInterval(() => {
        void overlay.refreshText();
      }, 250);
    }
    let unlisteners: Array<Promise<() => void>> = [];
    if (hasTauriApi()) {
      const unlistenSession = listen<SessionState>("session-state-changed", (event) => {
        applySessionState(event.payload);
      });
      const unlistenAsr = listen<AsrFinalText>("asr-final-text", (event) => {
        if (event.payload.error) {
          sessionErrorCode = event.payload.error_code;
          statusMessage = userErrorMessage(event.payload.error_code, event.payload.error);
          notifications.show(statusMessage, "error");
          if (shouldOpenSettingsForError(event.payload.error, event.payload.error_code)) {
            settingsNav.scrollToSettingsPanel(settingsPanelForError(event.payload.error, event.payload.error_code));
          }
          return;
        }
        const visibleWarning = event.payload.warning && !isQuietAsrWarningCode(event.payload.warning_code)
          ? event.payload.warning
          : null;
        if (visibleWarning) {
          notifications.show(visibleWarning, "warning");
        }
        lastSessionOutcome = {
          kind: "success",
          text: event.payload.text,
          warning: visibleWarning,
          warningCode: visibleWarning ? event.payload.warning_code : null,
          createdAt: Date.now(),
        };
        statusMessage = visibleWarning ?? t("sessionSucceeded");
        if (sessionPhase === "succeeded") scheduleSucceededIdleHint();
      });
      const unlistenOverlay = listen<OverlayText>("overlay-text", (event) => {
        overlay.applyText(event.payload.text);
      });
      const unlistenOverlayConfig = listen<OverlayConfig>("overlay-config", (event) => {
        overlay.applyConfig(event.payload.ui);
      });
      const unlistenStats = listen<StatsSnapshot>("usage-stats-updated", (event) => {
        if (!isOverlay && !isToast) {
          stats.apply(event.payload);
          void autoHotwords.refreshStatus();
        }
      });
      const unlistenAudioLevel = listen<AudioLevel>("audio-level", (event) => {
        audioLevel = clampAudioLevel(event.payload.level);
      });
      const unlistenClosePrompt = listen<CloseToTrayRequest>("close-to-tray-requested", (event) => {
        windows.showClosePrompt(event.payload);
      });
      const unlistenTrayUpdateCheck = listen("check-update-requested", () => {
        void updates.check(true);
      });
      unlisteners = [
        unlistenSession,
        unlistenAsr,
        unlistenOverlay,
        unlistenOverlayConfig,
        unlistenStats,
        unlistenAudioLevel,
        unlistenClosePrompt,
        unlistenTrayUpdateCheck,
      ];
      logFrontendEvent(`listeners registered mode=${frontendMode()}`);
    }
    return () => {
      if (overlayPoll !== undefined) window.clearInterval(overlayPoll);
      notifications.dispose();
      if (succeededIdleTimer !== undefined) window.clearTimeout(succeededIdleTimer);
      clearAutoSaveTimer();
      clearConfigSavedIndicatorTimer();
      overlay.dispose();
      window.removeEventListener("resize", refreshMainDensity);
      window.removeEventListener("resize", overlay.refreshLayout);
      window.removeEventListener("error", onError);
      window.removeEventListener("unhandledrejection", onUnhandledRejection);
      void Promise.all(unlisteners).then((disposers) => {
        for (const dispose of disposers) dispose();
      });
    };
  });

  $effect(() => {
    const fingerprint = configFingerprint(config);
    const shouldSave =
      fingerprint !== savedConfigFingerprint &&
      configLoaded &&
      hotkeyCapture.isIdle &&
      !isOverlay &&
      !isToast &&
      hasTauriApi();

    if (shouldSave) {
      scheduleAutoSaveConfig();
    } else if (fingerprint === savedConfigFingerprint) {
      clearAutoSaveTimer();
    }
  });

  function clearAutoSaveTimer() {
    if (autoSaveTimer !== undefined && browser) window.clearTimeout(autoSaveTimer);
    autoSaveTimer = undefined;
  }
  function clearConfigSavedIndicatorTimer() {
    if (configSavedIndicatorTimer !== undefined && browser) window.clearTimeout(configSavedIndicatorTimer);
    configSavedIndicatorTimer = undefined;
  }
  function markConfigSavedRecently() {
    if (!browser) return;
    clearConfigSavedIndicatorTimer();
    configSavedRecently = true;
    configSavedIndicatorTimer = window.setTimeout(() => {
      configSavedRecently = false;
      configSavedIndicatorTimer = undefined;
    }, 2400);
  }
  function canAutoSaveConfig() {
    return configLoaded && !isOverlay && !isToast && hasTauriApi() && hotkeyCapture.isIdle;
  }
  function scheduleAutoSaveConfig() {
    if (!canAutoSaveConfig()) return;
    clearAutoSaveTimer();
    autoSaveTimer = window.setTimeout(() => {
      autoSaveTimer = undefined;
      void autoSaveConfig();
    }, autoSaveDelayMs);
  }
  async function autoSaveConfig() {
    if (!canAutoSaveConfig() || !settingsDirty) return;
    if (saving) {
      scheduleAutoSaveConfig();
      return;
    }
    await persistConfig({ enforceAuth: false, focusErrors: false });
  }
  function refreshMainDensity() {
    if (isOverlay || isToast) {
      uiCompact = false;
      return;
    }
    uiCompact = window.innerHeight <= 820 || window.innerWidth <= 1260;
  }
  async function bootstrapApp() {
    const startedAt = performance.now();
    logFrontendEvent(`bootstrap started mode=${frontendMode()}`);
    try {
      await loadAll();
      await hydrateSession();
      void updates.maybeAutoCheck();
      logFrontendEvent(
        `bootstrap completed mode=${frontendMode()} elapsed_ms=${Math.round(performance.now() - startedAt)} config_exists=${configExists} recording=${recording}`,
      );
    } catch (error) {
      logFrontendError(`bootstrap failed: ${formatFrontendError(error)}`);
    }
  }
  function frontendMode() {
    return getFrontendMode(isOverlay, isToast);
  }
  function t(key: CopyKey, values: Record<string, string> = {}) {
    let value = copy[language][key];
    for (const [name, replacement] of Object.entries(values)) {
      value = value.replace(`{${name}}`, replacement);
    }
    return value;
  }

  function setLanguage(value: string) {
    if (value !== "zh-CN" && value !== "zh-TW" && value !== "en") return;
    language = value;
    localStorage.setItem("voxtype-language", value);
    syncTrayLanguage(value);
    if (
      statusMessage === copy["zh-CN"].bridgeLoading ||
      statusMessage === copy["zh-TW"].bridgeLoading ||
      statusMessage === copy.en.bridgeLoading
    ) {
      statusMessage = t("bridgeLoading");
    }
  }

  function syncTrayLanguage(value: Language) {
    void safeInvoke<void>("set_tray_language", { language: value }, true);
  }

  async function safeInvoke<T>(command: string, args?: Record<string, unknown>, quiet = false): Promise<T | null> {
    if (!hasTauriApi()) {
      if (!quiet) statusMessage = t("browserPreview");
      return null;
    }
    try {
      return await invoke<T>(command, args);
    } catch (error) {
      if (!quiet) statusMessage = userFacingInvokeFailure(command, error, t("operationFailedGeneric"));
      logFrontendError(`invoke failed command=${command}: ${formatFrontendError(error)}`);
      return null;
    }
  }
  async function toggleRecordingFromUi() {
    if (requireAsrAuthGate()) return;
    if (isSessionBusy()) return;
    if (!recording) lastSessionOutcome = null;
    const result = await safeInvoke<SessionState>("toggle_recording");
    if (result) applySessionState(result);
  }

  async function copyLastOutcomeText(text: string) {
    if (!text) return false;
    if (!hasTauriApi()) {
      statusMessage = t("browserPreview");
      notifications.show(statusMessage, "error");
      return false;
    }
    try {
      await invoke(copyRecentInputCommand, { text });
      statusMessage = t("lastOutcomeCopied");
      notifications.show(statusMessage, "success");
      return true;
    } catch (error) {
      statusMessage = userFacingInvokeFailure(copyRecentInputCommand, error, t("operationFailedGeneric"));
      logFrontendError(`copy last outcome text failed: ${formatFrontendError(error)}`);
      notifications.show(statusMessage, "error");
      return false;
    }
  }

  function isSessionBusy() {
    return isBlockingSessionPhase(sessionPhase);
  }

  function rememberSetupStatus(status: SetupStatus) {
    if (!browser || isOverlay || isToast) return;
    try {
      localStorage.setItem(setupStatusCacheKey, JSON.stringify(status));
    } catch {
      // 本地缓存只用于首屏体验，失败不影响真实检查。
    }
  }

  function applySetupStatus(status: SetupStatus) {
    setupStatus = status;
    setupStatusLoading = false;
    rememberSetupStatus(status);
  }

  function localSetupStatusFromConfig(configValue: AppConfig, devices = audioDevices): SetupStatus {
    return buildLocalSetupStatus(configValue, devices, setupStatus?.warnings ?? []);
  }

  function applyLoadedConfig(loaded: LoadedConfig) {
    config = loaded.data;
    savedConfigFingerprint = configFingerprint(loaded.data);
    configExists = loaded.exists;
  }

  async function loadAll() {
    logFrontendEvent(`loadAll started mode=${frontendMode()}`);
    if (!isOverlay && !isToast && !setupStatus) setupStatusLoading = true;
    const [snapshotResult, configResult, statsResult, devicesResult, setupResult, localDataResult] = await Promise.all([
      safeInvoke<AppSnapshot>("get_app_snapshot"),
      safeInvoke<LoadedConfig>("load_app_config"),
      safeInvoke<StatsSnapshot>("get_usage_stats"),
      safeInvoke<AudioDeviceInfo[]>("list_audio_input_devices"),
      safeInvoke<SetupStatus>("get_setup_status"),
      safeInvoke<LocalDataStatus>("get_local_data_status"),
    ]);
    await autoHotwords.refreshStatus();
    const loadedAny = Boolean(snapshotResult || configResult || statsResult || devicesResult || setupResult);
    if (snapshotResult) snapshot = snapshotResult;
    if (configResult) {
      applyLoadedConfig(configResult);
      configLoaded = true;
      const setupMessage = configSetupMessage(configResult);
      if (setupMessage) {
        statusMessage = setupMessage;
        if (!isOverlay && !isToast && requiresAsrAuth(configResult.data, configResult.exists)) {
          settingsNav.showApiConfigIntro();
        }
      }
    }
    if (statsResult) stats.apply(statsResult);
    if (localDataResult) privacy.apply(localDataResult);
    if (devicesResult) audioDevices = devicesResult;
    if (!configResult && hasTauriApi() && !isOverlay && !isToast) configLoaded = true;
    if (setupResult) {
      applySetupStatus(setupResult);
    } else if (!setupStatus && configResult) {
      setupStatus = localSetupStatusFromConfig(configResult.data, devicesResult ?? audioDevices);
    }
    if (!isOverlay && !isToast) setupStatusLoading = false;
    if ((snapshotResult || configResult || statsResult) && !configSetupMessage(configResult)) {
      statusMessage = t("bridgeConnected");
    }
    logFrontendEvent(
      `loadAll completed mode=${frontendMode()} snapshot=${Boolean(snapshotResult)} config_loaded=${Boolean(configResult)} config_exists=${configResult?.exists ?? false} stats_records=${statsResult?.history.length ?? 0} audio_devices=${devicesResult?.length ?? 0} setup_ready=${setupResult?.ready ?? false} auto_hotword_entries=${autoHotwords.status?.entry_count ?? 0}`,
    );
    return loadedAny;
  }
  async function hydrateSession() {
    logFrontendEvent(`hydrateSession started mode=${frontendMode()}`);
    const result = await safeInvoke<SessionState>("get_session_state");
    if (result) applySessionState(result);
    logFrontendEvent(
      `hydrateSession completed mode=${frontendMode()} state_loaded=${Boolean(result)} recording=${result?.recording ?? false}`,
    );
  }

  function applySessionState(state: SessionState) {
    recording = state.recording;
    sessionPhase = state.phase ?? (state.recording ? "recording" : "idle");
    sessionErrorCode = state.error_code;
    if (sessionPhase === "starting" || sessionPhase === "recording") lastSessionOutcome = null;
    if (sessionPhase !== "succeeded" && succeededIdleTimer !== undefined) {
      window.clearTimeout(succeededIdleTimer);
      succeededIdleTimer = undefined;
    }
    if (!state.recording) audioLevel = 0;
    if (state.phase === "failed" && state.error_code) {
      statusMessage = userErrorMessage(state.error_code, state.message);
      if (shouldOpenSettingsForError(state.message, state.error_code)) {
        settingsNav.scrollToSettingsPanel(settingsPanelForError(state.message, state.error_code));
      }
      return;
    }
    if (isConfigError(state.message)) {
      statusMessage = userErrorMessage(state.error_code, state.message);
      settingsNav.scrollToSettingsPanel(settingsPanelForError(state.message, state.error_code));
      return;
    }
    statusMessage = state.phase === "failed" && state.message ? userErrorMessage(state.error_code, state.message) : sessionPhaseMessage(sessionPhase);
    if (sessionPhase === "succeeded") scheduleSucceededIdleHint();
  }
  function scheduleSucceededIdleHint() {
    if (succeededIdleTimer !== undefined) window.clearTimeout(succeededIdleTimer);
    succeededIdleTimer = window.setTimeout(() => {
      if (sessionPhase !== "succeeded") return;
      sessionPhase = "idle";
      statusMessage = sessionPhaseMessage("idle");
      succeededIdleTimer = undefined;
    }, 2000);
  }
  function sessionPhaseMessage(phase: SessionPhase) {
    const hotkey = formatHotkey(snapshot.hotkey);
    return t(sessionPhaseMessageKey(phase), { hotkey });
  }

  async function refreshStats() {
    const result = await safeInvoke<StatsSnapshot>("get_usage_stats", undefined, true);
    if (result) stats.apply(result);
  }

  async function persistConfig(options: PersistConfigOptions = {}) {
    const { enforceAuth = true, focusErrors = true } = options;
    if (saving) return null;
    const configToSave = clonePlain(config);
    const saveFingerprint = configFingerprint(configToSave);
    saving = true;
    try {
      validationErrors = {};
      const hotkeyError = hotkeyCapture.validate(configToSave.hotkey);
      if (hotkeyError) {
        validationErrors = { hotkey: hotkeyError };
        statusMessage = hotkeyError;
        if (focusErrors) settingsNav.scrollToSettingsPanel("settings-output");
        return null;
      }
      if (enforceAuth && !requireAuthFields(focusErrors, focusErrors)) return null;
      if (!hasTauriApi()) {
        statusMessage = t("browserPreview");
        return null;
      }
      const result = await invoke<LoadedConfig>("save_app_config", { config: configToSave });
      if (result) {
        const resultFingerprint = configFingerprint(result.data);
        const currentFingerprint = configFingerprint(config);
        savedConfigFingerprint = resultFingerprint;
        if (currentFingerprint === saveFingerprint) config = result.data;
        snapshot = { ...snapshot, hotkey: result.data.hotkey };
        syncSetupStatusFromConfig(result.data);
        configExists = result.exists;
        configLoaded = true;
        statusMessage = t("configSaved");
        markConfigSavedRecently();
      }
      return result;
    } catch (error) {
      const saveError = parseConfigSaveError(error);
      const errors = saveError.errors ?? [];
      validationErrors = validationErrorMap(errors);
      if (focusErrors) settingsNav.focusFirstValidationError(errors);
      statusMessage = saveError.message || t("validationFailed");
      logFrontendError(`save config failed: ${formatFrontendError(error)}`);
      return null;
    } finally {
      saving = false;
    }
  }
  function parseConfigSaveError(error: unknown): ConfigSaveError {
    if (typeof error === "object" && error !== null) {
      const maybeError = error as { message?: unknown; errors?: unknown };
      return {
        message: typeof maybeError.message === "string" ? maybeError.message : t("validationFailed"),
        errors: Array.isArray(maybeError.errors) ? (maybeError.errors as ConfigValidationError[]) : [],
      };
    }
    return {
      message: typeof error === "string" ? error : t("validationFailed"),
      errors: [],
    };
  }
  function fieldError(field: string) {
    return validationErrors[field] ?? "";
  }
  function syncSetupStatusFromConfig(nextConfig: AppConfig) {
    const currentStatus = setupStatus ?? localSetupStatusFromConfig(nextConfig);
    applySetupStatus(mergeSetupStatusFromConfig(nextConfig, currentStatus));
  }
  function authFieldErrors() {
    const errors: Record<string, string> = {};
    if (!config.auth.app_key.trim()) errors["auth.app_key"] = t("requiredField");
    if (!config.auth.access_key.trim()) errors["auth.access_key"] = t("requiredField");
    return errors;
  }
  function clearAuthFieldErrors() {
    const next = { ...validationErrors };
    delete next["auth.app_key"];
    delete next["auth.access_key"];
    validationErrors = next;
  }
  function requireAuthFields(showNotice = true, focusTarget = true) {
    const errors = authFieldErrors();
    if (Object.keys(errors).length === 0) {
      clearAuthFieldErrors();
      return true;
    }
    validationErrors = { ...validationErrors, ...errors };
    statusMessage = authGateMessage();
    if (focusTarget) settingsNav.focusAsrAuthSettings();
    if (showNotice) notifications.show(statusMessage, "warning");
    return false;
  }
  async function testAsrConfig() {
    if (testingAsr) return;
    if (!requireAuthFields()) return;
    testingAsr = true;
    asrConnectionStatus = "testing";
    try {
      const result = await safeInvoke<ConnectionTestResult>("test_asr_config", { config: clonePlain(config) });
      if (result) {
        asrConnectionStatus = "tested_ok";
        asrTestedConfigFingerprint = asrConfigFingerprint();
        statusMessage = t("asrTestSucceeded");
        notifications.show(statusMessage, "success");
      } else if (statusMessage) {
        asrConnectionStatus = "tested_failed";
        asrTestedConfigFingerprint = asrConfigFingerprint();
        notifications.show(statusMessage, "error");
      }
    } finally {
      testingAsr = false;
    }
  }
  async function testLlmConfig() {
    if (testingLlm) return;
    testingLlm = true;
    try {
      const result = await safeInvoke<ConnectionTestResult>("test_llm_config", { config: clonePlain(config) });
      if (result) {
        let savedStrategy = false;
        if (
          typeof result.thinking_strategy === "string" &&
          result.thinking_strategy &&
          config.llm_post_edit.thinking_strategy !== result.thinking_strategy
        ) {
          config.llm_post_edit.thinking_strategy = result.thinking_strategy;
          savedStrategy = Boolean(await persistConfig({ enforceAuth: false, focusErrors: false }));
        }
        statusMessage =
          typeof result.elapsed_ms === "number" && savedStrategy
            ? t("llmTestSucceededWithLatencyAndStrategy", {
                ms: formatNumber(result.elapsed_ms),
                strategy: result.thinking_strategy ?? "",
              })
            : typeof result.elapsed_ms === "number"
              ? t("llmTestSucceededWithLatency", { ms: formatNumber(result.elapsed_ms) })
              : t("llmTestSucceeded");
        notifications.show(statusMessage, "success");
      } else if (statusMessage) {
        notifications.show(statusMessage, "error");
      }
    } finally {
      testingLlm = false;
    }
  }
  async function testScreenContext() {
    if (testingScreenContext) return;
    testingScreenContext = true;
    screenContextTestResult = null;
    try {
      const result = await safeInvoke<ScreenContextTestResult>("test_screen_context", { config: clonePlain(config) });
      if (result) {
        screenContextTestResult = result;
        statusMessage = t("screenContextTestSucceeded", {
          chars: formatNumber(result.text_chars),
          ms: formatNumber(result.elapsed_ms),
        });
        notifications.show(statusMessage, result.warning ? "warning" : "success");
      } else if (statusMessage) {
        notifications.show(statusMessage, "error");
      }
    } finally {
      testingScreenContext = false;
    }
  }
  function optionEnabledNotice(key: SoftConfigNoticeKey, enabled: boolean) {
    if (!enabled) return "";
    if (key === "middle_mouse_enabled" || key === "right_alt_enabled") return t("extraTriggerEnabledNotice");
    if (key === "enable_recent_context") return t("recentContextEnabledNotice");
    return "";
  }
  function maybeShowOptionEnabledNotice(key: SoftConfigNoticeKey, enabled: boolean) {
    const notice = optionEnabledNotice(key, enabled);
    if (notice) notifications.show(notice, "info");
  }
  function triggerLabel(enabled: boolean) {
    return enabled ? t("enabled") : t("disabled");
  }
  function micBarHeight(index: number) {
    return getMicBarHeight(recording, audioLevel, index);
  }
  function micBarOpacity(index: number) {
    return getMicBarOpacity(recording, audioLevel, index);
  }
  function currentAudioDevice() {
    if (audioDevices.length === 0) return null;
    if (config.audio.input_device !== null && config.audio.input_device !== undefined) {
      const configured = audioDevices.find((device) => device.index === config.audio.input_device);
      if (configured) return configured;
    }
    return audioDevices.find((device) => device.is_default) ?? audioDevices[0];
  }
  async function refreshSetupStatus(showLoading = true) {
    if (showLoading || !setupStatus) setupStatusLoading = true;
    const [devicesResult, setupResult] = await Promise.all([
      safeInvoke<AudioDeviceInfo[]>("list_audio_input_devices", undefined, true),
      safeInvoke<SetupStatus>("get_setup_status", undefined, true),
    ]);
    if (devicesResult) audioDevices = devicesResult;
    if (setupResult) {
      applySetupStatus(setupResult);
    } else if (!setupStatus) {
      setupStatus = localSetupStatusFromConfig(config, devicesResult ?? audioDevices);
    }
    setupStatusLoading = false;
  }
  function setupStatusItems(): SetupStatusItem[] {
    const localStatus = localSetupStatusFromConfig(config);
    return buildSetupStatusItems({
      loading: setupStatusLoading,
      configLoaded,
      config,
      setupStatus,
      localStatus,
      audioDevices,
      asrStatus: currentAsrConnectionStatus(setupStatus ?? localStatus),
      triggerText: formatEnabledTriggers(),
      t,
    });
  }
  function setupWarningCount() {
    if (setupStatusLoading && !setupStatus) return 0;
    return setupStatusItems().filter((item) => !item.ok).length;
  }
  function setupIsReady() {
    if (setupStatusLoading && !setupStatus) return false;
    const status = setupStatus ?? localSetupStatusFromConfig(config);
    const baseReady = status.ready;
    return baseReady && currentAsrConnectionStatus(status) !== "tested_failed";
  }
  function setupActionText(action: string) {
    return getSetupActionText(action, t);
  }
  function handleSetupAction(action: string) {
    if (action === "audio") void refreshSetupStatus();
    if (action === "privacy") {
      settingsNav.selectSection("Privacy");
      void privacy.refreshStatus();
      return;
    }
    const targetId =
      action === "asr_auth"
        ? "settings-auth"
        : action === "audio"
          ? "settings-audio"
          : action === "typing"
            ? "settings-output"
            : "settings-output";
    settingsNav.scrollToSettingsPanel(targetId);
  }
  function pasteMethodLabel(value: string) {
    return getPasteMethodLabel(value, t);
  }
  function asrConfigFingerprint(configValue = config) {
    return buildAsrConfigFingerprint(configValue);
  }
  function currentAsrConnectionStatus(status: SetupStatus | null = null): AsrConnectionStatus {
    return getCurrentAsrConnectionStatus({
      status,
      authReady: hasAuth(),
      testingAsr,
      currentFingerprint: asrConfigFingerprint(),
      testedFingerprint: asrTestedConfigFingerprint,
      asrConnectionStatus,
    });
  }
  function asrConnectionStatusText(status: AsrConnectionStatus) {
    return getAsrConnectionStatusText(status, t);
  }
  function asrConnectionStatusOk(status: AsrConnectionStatus) {
    return isAsrConnectionStatusOk(status);
  }
  function formatEnabledTriggers() {
    return getEnabledTriggersText(config, snapshot.hotkey, t, formatHotkey);
  }
  function micStatusText() {
    const device = currentAudioDevice();
    if (!device) return !configLoaded && setupStatus?.has_audio_device ? t("setupMicDetected") : t("micUnavailable");
    return recording
      ? t("micMonitoring", { device: device.name })
      : t("micConnected", { device: device.name });
  }
  function sidebarMicStatusText() {
    return currentAudioDevice() || (!configLoaded && setupStatus?.has_audio_device)
      ? t("sidebarMicConnected")
      : t("sidebarMicUnavailable");
  }
  function updateHotwords(value: string) {
    config.context.hotwords = normalizeHotwords(value);
  }

  function effectiveHotwords() {
    return mergedEffectiveHotwords(config);
  }

  function hotwordCount() {
    return countManualHotwords(config);
  }

  function tidyHotwords() {
    config.context.hotwords = dedupeHotwords(config.context.hotwords);
    notifications.show(t("hotwordsTidied"), "success");
  }

  function clearHotwords() {
    if (!browser || window.confirm(t("clearHotwordsConfirm"))) {
      config.context.hotwords = [];
      notifications.show(t("hotwordsCleared"), "success");
    }
  }

  function updatePromptContext(value: string) {
    config.context.prompt_context = value
      .split("\n")
      .map((text) => text.trim())
      .filter(Boolean)
      .map((text) => ({ text }));
  }

  function restoreDefaultLlmPrompt() {
    config.llm_post_edit.system_prompt = fallbackConfig.llm_post_edit.system_prompt;
    config.llm_post_edit.user_prompt_template = fallbackConfig.llm_post_edit.user_prompt_template;
    notifications.show(t("defaultPromptRestored"), "success");
  }

  function previewFinalPrompt() {
    const sampleText = t("promptPreviewSampleText");
    promptPreviewText = buildFinalPromptPreview(config, sampleText, effectiveHotwords(), {
      dictionary: t("promptPreviewUserDictionary"),
      dictionaryPurpose: t("promptPreviewUserDictionaryPurpose"),
      dictionaryEnd: t("promptPreviewUserDictionaryEnd"),
      context: t("promptPreviewContextTitle"),
      contextPurpose: t("promptPreviewContextPurpose"),
      contextEnd: t("promptPreviewContextEnd"),
      recentContext: t("promptPreviewRecentContextTitle"),
      recentContextPurpose: t("promptPreviewRecentContextPurpose"),
      recentContextEnd: t("promptPreviewRecentContextEnd"),
      recentContextPlaceholder: t("promptPreviewRecentContextPlaceholder"),
      screenOcrContext: t("promptPreviewScreenOcrTitle"),
      screenOcrPurpose: t("promptPreviewScreenOcrPurpose"),
      screenOcrEnd: t("promptPreviewScreenOcrEnd"),
      screenOcrPlaceholder: t("promptPreviewScreenOcrPlaceholder"),
      referenceRules: t("promptPreviewReferenceRules"),
      systemPrompt: t("systemPrompt"),
      userPromptTemplate: t("userPromptTemplate"),
      empty: t("promptPreviewEmpty"),
      summaryTitle: t("promptPreviewSummaryTitle"),
      sceneContextSummary: t("promptPreviewSceneContextSummary"),
      recentContextPolicyDisabled: t("promptPreviewRecentContextPolicyDisabled"),
      recentContextPolicyEnabled: t("promptPreviewRecentContextPolicyEnabled"),
      recentContextPolicyNeedsLocal: t("promptPreviewRecentContextPolicyNeedsLocal"),
      screenOcrPolicyEnabled: t("promptPreviewScreenOcrPolicyEnabled"),
      screenOcrPolicyDisabled: t("promptPreviewScreenOcrPolicyDisabled"),
      actualPromptTitle: t("promptPreviewActualPromptTitle"),
    });
  }

  function closePromptPreview() {
    promptPreviewText = "";
  }

  async function copyPromptPreview() {
    if (!browser || !promptPreviewText) return;
    try {
      await navigator.clipboard.writeText(promptPreviewText);
      notifications.show(t("promptPreviewCopied"), "success");
    } catch (err) {
      logFrontendError(`copy prompt preview failed: ${formatFrontendError(err)}`);
      notifications.show(t("operationFailedGeneric"), "error");
    }
  }

  function setInputDevice(value: string | number | null) {
    if (value === null || value === "") {
      config.audio.input_device = null;
      return;
    }
    config.audio.input_device = Number(value);
  }

  function formatNumber(value: number) {
    return formatNumberForLanguage(value, language);
  }
  function inputStatus(): "idle" | "listening" | "error" {
    if (sessionPhase === "failed" || isErrorStatus(statusMessage)) return "error";
    return recording || isSessionBusy() ? "listening" : "idle";
  }
  function inputStatusLabel() {
    const status = inputStatus();
    if (status === "error") return isConfigError(statusMessage) ? t("setupRequired") : t("inputError");
    if (sessionPhase === "starting") return t("sessionStarting");
    if (sessionPhase === "stopping") return t("sessionStopping");
    if (sessionPhase === "waiting_final_result") return t("sessionWaitingFinal");
    if (sessionPhase === "post_editing") return t("sessionPostEditing");
    if (sessionPhase === "pasting") return t("sessionPasting");
    if (sessionPhase === "succeeded") return t("lastOutcomeSuccessTitle");
    return recording ? t("recordingPreview") : t("idle");
  }
  function inputStatusDesc() {
    const status = inputStatus();
    if (status === "error") return statusMessage;
    return sessionPhaseMessage(sessionPhase);
  }
  function configSaveState(): ConfigSaveState {
    if (!configLoaded || !hasTauriApi() || isOverlay || isToast) return "idle";
    if (saving) return "saving";
    if (settingsDirty) return "pending";
    if (configSavedRecently) return "saved";
    return "idle";
  }
  function formatSavedHours(hours: number) {
    return formatSavedHoursForLanguage(hours, language);
  }
  function hasAuth(configValue = config) {
    return configHasAuth(configValue);
  }
  function hasLlmApiConfig(configValue = config) {
    return configHasLlmApiConfig(configValue);
  }
  function llmApiStatusText() {
    return hasLlmApiConfig() ? t("llmApiConfigured") : t("llmApiMissing");
  }
  function requiresAsrAuth(configValue?: AppConfig, exists?: boolean) {
    return configRequiresAsrAuth({
      configLoaded,
      setupStatus,
      config,
      configExists,
      targetConfig: configValue,
      targetExists: exists,
    });
  }
  function authGateMessage() {
    return !configExists ? t("setupMissingFile") : t("authGateNotice");
  }
  function setupRequiredMessage() {
    return !configExists ? t("setupMissingFile") : t("setupMissingAuth");
  }
  function requireAsrAuthGate(showNotice = true) {
    if (!requiresAsrAuth()) return false;
    statusMessage = authGateMessage();
    settingsNav.focusAsrAuthSettings();
    if (showNotice) notifications.show(statusMessage, "warning");
    return true;
  }
  function configSetupMessage(loaded: LoadedConfig | null) {
    return getConfigSetupMessage(loaded, t);
  }
  function userErrorDetail(code: string | null | undefined, fallback = ""): UserErrorDetail {
    return getUserErrorDetail(code, fallback, language, t);
  }
  function userErrorMessage(code: string | null | undefined, fallback = "") {
    return getUserErrorMessage(code, fallback, language, t);
  }
  function activeUserErrorDetail() {
    if (inputStatus() !== "error") return null;
    return userErrorDetail(sessionErrorCode, statusMessage);
  }
  function activeUserErrorActions() {
    if (inputStatus() !== "error") return [];
    if (requiresAsrAuth() || isConfigError(statusMessage)) return actionsForUserError("ASR_AUTH_MISSING");
    return actionsForUserError(sessionErrorCode);
  }
  function handleUserErrorAction(action: UserErrorAction) {
    switch (action) {
      case "retry_recording":
        if (!isSessionBusy() && !requiresAsrAuth()) void toggleRecordingFromUi();
        break;
      case "open_api_config":
        settingsNav.scrollToSettingsPanel("settings-auth");
        break;
      case "open_options":
        settingsNav.selectSection("Options");
        break;
      case "open_setup_guide":
        void openSetupGuide();
        break;
      case "copy_diagnostic_report":
        void diagnostics.copyReport();
        break;
      case "open_log":
        void diagnostics.openLog();
        break;
    }
  }
  function isErrorStatus(message: string) {
    return isUserErrorStatus(message);
  }
  function appShellProps() {
    return {
      uiCompact,
      selectedSection: settingsNav.selectedSection,
      language,
      recording,
      configSaveState: configSaveState(),
      inputStatus: inputStatus(),
      inputStatusLabel: inputStatusLabel(),
      inputStatusDesc: inputStatusDesc(),
      micBars,
      snapshotHotkey: snapshot.hotkey,
      requiresAsrAuth: requiresAsrAuth(),
      t,
      formatHotkey,
      micStatusText,
      sidebarMicStatusText,
      micBarHeight,
      micBarOpacity,
      onSelectSection: settingsNav.selectSection,
      onSetLanguage: setLanguage,
      onClose: windows.close,
      onMinimize: windows.minimize,
      onToggleMaximize: windows.toggleMaximize,
    };
  }
  function appContentProps() {
    return {
      selectedSection: settingsNav.selectedSection,
      stats: stats.snapshot,
      t,
      uiCompact,
      recording,
      saving,
      inputStatus: inputStatus(),
      inputStatusLabel: inputStatusLabel(),
      inputStatusDesc: inputStatusDesc(),
      requiresAsrAuth: requiresAsrAuth(),
      setupRequiredMessage,
      activeErrorDetail: activeUserErrorDetail(),
      activeErrorActions: activeUserErrorActions(),
      lastSessionOutcome,
      sessionBusy: isSessionBusy(),
      snapshotHotkey: snapshot.hotkey,
      chineseTypingCharsPerMinute,
      configExists,
      setupChecking: setupStatusLoading && !setupStatus,
      setupStatusReady: setupIsReady(),
      setupStatusItems: setupStatusItems(),
      setupWarnings: localizeSetupWarnings(setupStatus?.warnings ?? [], t),
      setupWarningCount: setupWarningCount(),
      testingAsr,
      testingLlm,
      testingScreenContext,
      screenContextTestResult,
      hotkeyCaptureState: hotkeyCapture.state,
      hotkeyValidationMessage: hotkeyCapture.validationMessage,
      overlayColorPresets,
      overlayOpacityPresets,
      audioDevices,
      updateStatus: updates.status,
      checkingUpdate: updates.checking,
      installingUpdate: updates.installing,
      openingLog: diagnostics.openingLog,
      copyingDiagnosticReport: diagnostics.copyingReport,
      generatingAutoHotwords: autoHotwords.generating,
      clearingAutoHotwordHistory: autoHotwords.clearingHistory,
      autoHotwordError: autoHotwords.error,
      showAutoHotwordDetails: autoHotwords.showDetails(),
      hasLlmApiConfig: hasLlmApiConfig(),
      hotwordCount: hotwordCount(),
      acceptedAutoHotwordCount: autoHotwords.acceptedCount(),
      selectedAutoHotwordCount: autoHotwords.selectedCount(),
      autoHotwordStatusText: autoHotwords.statusText(),
      llmApiStatusText: llmApiStatusText(),
      fieldError,
      candidateConfidenceLabel,
      formatHotkey,
      formatNumber,
      formatHours,
      formatSavedHours,
      weeklySavedHours: stats.weeklySavedHours,
      usageTipText: stats.usageTipText,
      triggerLabel,
      setupActionText,
      overlayBackgroundRgb: overlay.backgroundRgb,
      overlayOpacity: overlay.opacity,
      overlayTextColor: overlay.textColor,
      overlayPresetActive: overlay.presetActive,
      overlayOpacityPresetActive: overlay.opacityPresetActive,
      overlayOpacityLabel,
      updatePanelTitle: updates.panelTitle,
      updatePanelDescription: updates.panelDescription,
      updateMetaText: updates.metaText,
      historySummaryCards: stats.historySummaryCards,
      recentSevenDayDisplayRows: stats.recentSevenDayDisplayRows,
      privacyStatus: privacy.status,
      privacyClearingRecentContext: privacy.clearingRecentContext,
      privacyClearingAutoHotwordHistory: privacy.clearingAutoHotwordHistory,
      privacyClearingUsageStats: privacy.clearingUsageStats,
      onOpenSettings: openSettings,
      onOpenSetupGuide: openSetupGuide,
      onUserErrorAction: handleUserErrorAction,
      onCopyLastOutcomeText: copyLastOutcomeText,
      onToggleRecording: toggleRecordingFromUi,
      onSelectSection: settingsNav.selectSection,
      onUpdateHotwords: updateHotwords,
      onTidyHotwords: tidyHotwords,
      onClearHotwords: clearHotwords,
      onUpdatePromptContext: updatePromptContext,
      onOptionEnabledNotice: maybeShowOptionEnabledNotice,
      onRestoreDefaultPrompt: restoreDefaultLlmPrompt,
      onPreviewFinalPrompt: previewFinalPrompt,
      onOpenLlmApiSettings: settingsNav.openLlmApiSettings,
      onGenerateAutoHotwords: autoHotwords.generate,
      onClearAutoHotwordHistory: autoHotwords.clearHistory,
      onRefreshAutoHotwordStatus: autoHotwords.refreshStatus,
      onUpdateAcceptedAutoHotwords: autoHotwords.updateAccepted,
      onTidyAcceptedAutoHotwords: autoHotwords.tidyAccepted,
      onClearAcceptedAutoHotwords: autoHotwords.clearAccepted,
      onApplySelectedAutoHotwords: autoHotwords.applySelected,
      onScrollToSettingsPanel: settingsNav.scrollToSettingsPanel,
      onRefreshSetupStatus: refreshSetupStatus,
      onSetupAction: handleSetupAction,
      onOpenDoubaoAsrDocs: openDoubaoAsrDocs,
      onTestAsrConfig: testAsrConfig,
      onTestLlmConfig: testLlmConfig,
      onTestScreenContext: testScreenContext,
      onHotkeyKeydown: hotkeyCapture.handleKeydown,
      onBeginHotkeyCapture: hotkeyCapture.beginCapture,
      onApplyOverlayPreset: overlay.applyPreset,
      onApplyOverlayOpacity: overlay.applyOpacity,
      onSetInputDevice: setInputDevice,
      onCheckUpdate: updates.check,
      onDownloadLatestUpdate: updates.downloadLatest,
      onOpenLog: diagnostics.openLog,
      onCopyDiagnosticReport: diagnostics.copyReport,
      onRefreshPrivacyStatus: privacy.refreshStatus,
      onClearPrivacyRecentContext: privacy.clearRecentContext,
      onClearPrivacyAutoHotwordHistory: privacy.clearAutoHotwordHistory,
      onClearPrivacyUsageStats: privacy.clearUsageStats,
    };
  }
  function openSettings() {
    settingsNav.scrollToSettingsPanel("settings-auth");
  }
  async function openSetupGuide() {
    await safeInvoke<void>("open_setup_guide");
  }
  async function openDoubaoAsrDocs() {
    await safeInvoke<void>("open_doubao_asr_docs");
  }

  return {
    get isOverlay() { return isOverlay; },
    get isToast() { return isToast; },
    get recording() { return recording; },
    get overlayMode() { return overlay.mode; },
    get overlayFontSize() { return overlay.fontSize; },
    get overlayDisplayLines() { return overlay.displayLines; },
    get overlayTextElement() { return overlay.textElement; },
    set overlayTextElement(value: HTMLDivElement | null) { overlay.textElement = value; },
    get overlayRootStyle() { return overlay.rootStyle; },
    get toastTitle() { return t("startupToastTitle"); },
    get toastHint() { return t("startupToastHint").replace("{hotkey}", formatHotkey(toastHotkey)); },
    get actionNotice() { return notifications.message; },
    get actionNoticeKind() { return notifications.kind; },
    get actionNoticeActionLabel() { return notifications.actionLabel; },
    get actionNoticeActionBusyLabel() { return notifications.actionBusyLabel; },
    get actionNoticeActionBusy() { return notifications.actionBusy; },
    get closePromptVisible() { return windows.closePromptVisible; },
    get closePromptTitle() { return t("closePromptTitle"); },
    get closePromptBody() { return t("closePromptBody"); },
    get closePromptGotItLabel() { return t("closePromptGotIt"); },
    get closePromptDontShowAgainLabel() { return t("closePromptDontShowAgain"); },
    get closePromptExitLabel() { return t("closePromptExit"); },
    get promptPreviewVisible() { return Boolean(promptPreviewText); },
    get promptPreviewTitle() { return t("promptPreviewDialogTitle"); },
    get promptPreviewText() { return promptPreviewText; },
    get promptPreviewCopyLabel() { return t("promptPreviewCopy"); },
    get promptPreviewCloseLabel() { return t("windowClose"); },
    get config() { return config; },
    set config(value: AppConfig) { config = value; },
    get autoHotwordCandidates() { return autoHotwords.candidates; },
    set autoHotwordCandidates(value: SelectableHotwordCandidate[]) { autoHotwords.candidates = value; },
    appShellProps,
    appContentProps,
    runActionNoticeAction: notifications.runAction,
    overlayMeterBarHeight: overlay.meterBarHeight,
    overlayMeterBarOpacity: overlay.meterBarOpacity,
    closeWindowWithoutFuturePrompt: windows.closeWithoutFuturePrompt,
    exitFromClosePrompt: windows.exitFromPrompt,
    confirmClosePrompt: windows.confirmClosePrompt,
    closePromptPreview,
    copyPromptPreview,
  };
}
