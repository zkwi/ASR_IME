import { onMount } from "svelte";
import { browser } from "$app/environment";
import { createAutoHotwordsController } from "$lib/app/autoHotwordsController.svelte";
import { createConfigController } from "$lib/app/configController.svelte";
import { createDiagnosticsController } from "$lib/app/diagnosticsController.svelte";
import { createHotkeyCaptureController } from "$lib/app/hotkeyCaptureController.svelte";
import { createLlmAutoAdaptController } from "$lib/app/llmAutoAdaptController.svelte";
import { createNotificationController } from "$lib/app/notificationController.svelte";
import { createOverlayController } from "$lib/app/overlayController.svelte";
import { createPrivacyController } from "$lib/app/privacyController.svelte";
import { createSettingsNavigationController } from "$lib/app/settingsNavigationController.svelte";
import {
  disposeNativeEventController,
  registerNativeEventController,
} from "$lib/app/nativeEventController.svelte";
import { createSessionController } from "$lib/app/sessionController.svelte";
import { createSetupController } from "$lib/app/setupController.svelte";
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
import { DOUBAO_AUTH_MODE_AGENT_PLAN, isAliyunAsrProvider } from "$lib/utils/asrProvider";
import { clonePlain, configFingerprint } from "$lib/utils/config";
import {
  canEditLoadedConfig,
  configLoadStateForResult,
  shouldProtectUnsavedChanges,
  type ConfigLoadState,
} from "$lib/utils/configPersistence";
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
  appendLlmTestRecord,
  readLlmTestHistory,
  saveLlmTestHistory,
  summarizeLlmTestHistory,
  type LlmTestRecord,
} from "$lib/utils/llmTestHistory";
import {
  sessionPhaseMessageKey,
  startsNewRecordingSession,
} from "$lib/utils/sessionState";
import { userFacingInvokeFailure } from "$lib/utils/userFacingErrors";
import { invokeErrorCode } from "$lib/utils/statusCodes";
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
import type {
  AppConfig,
  AppSnapshot,
  AsrConnectionStatus,
  AudioDeviceInfo,
  AudioQualityDiagnostic,
  ConfigMigrationCandidate,
  LastSessionOutcome,
  LoadedConfig,
  LocalDataStatus,
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
const configMigrationDismissedPrefix = "voxtype-config-migration-dismissed:";

export function createVoxTypeController() {

  let snapshot = $state<AppSnapshot>(fallbackSnapshot);
  let config = $state<AppConfig>(clonePlain(fallbackConfig));
  let savedConfigFingerprint = $state(configFingerprint(fallbackConfig));
  let settingsDirty = $derived(configFingerprint(config) !== savedConfigFingerprint);
  let recording = $state(false);
  let sessionPhase = $state<SessionPhase>("idle");
  let sessionErrorCode = $state<string | null>(null);
  let lastSessionOutcome = $state<LastSessionOutcome>(null);
  let lastAudioQualityDiagnostic = $state<AudioQualityDiagnostic | null>(null);
  let language = $state<Language>("zh-CN");
  let statusMessage = $state(copy["zh-CN"].bridgeLoading);
  let promptPreviewText = $state("");
  let configExists = $state(true);
  let configLoadState = $state<ConfigLoadState>("not_loaded");
  let configLoaded = $derived(canEditLoadedConfig(configLoadState));
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
    t,
  });
  let uiCompact = $state(false);
  let setupStatus = $state<SetupStatus | null>(readCachedSetupStatus(browser));
  let testingAsr = $state(false);
  let asrConnectionStatus = $state<AsrConnectionStatus>("missing_auth");
  let asrTestedConfigFingerprint = $state("");
  let testingLlm = $state(false);
  let llmTestStatusMessage = $state("");
  let llmTestHistory = $state<LlmTestRecord[]>(browser ? readLlmTestHistory(localStorage) : []);
  let llmAutoAdaptTestedFingerprint = $state("");
  let testingScreenContext = $state(false);
  let screenContextTestResult = $state<ScreenContextTestResult | null>(null);
  let validationErrors = $state<Record<string, string>>({});
  let llmAutoAdapt: ReturnType<typeof createLlmAutoAdaptController>;
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
  let succeededIdleTimer: number | undefined;
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
  const configController = createConfigController({
    autoSaveDelayMs,
    t,
    getConfig: () => config,
    setConfig: (value) => {
      config = value;
    },
    setSavedConfigFingerprint: (fingerprint) => {
      savedConfigFingerprint = fingerprint;
    },
    getSettingsDirty: () => settingsDirty,
    getConfigLoaded: () => configLoaded,
    setConfigExists: (value) => {
      configExists = value;
    },
    setSnapshotHotkey: (hotkey) => {
      snapshot = { ...snapshot, hotkey };
    },
    getValidationErrors: () => validationErrors,
    setValidationErrors: (errors) => {
      validationErrors = errors;
    },
    validateHotkey: hotkeyCapture.validate,
    requireAuthFields,
    syncSetupStatusFromConfig,
    scrollToSettingsPanel: settingsNav.scrollToSettingsPanel,
    focusFirstValidationError: settingsNav.focusFirstValidationError,
    setStatusMessage: (message) => {
      statusMessage = message;
    },
    hasTauriApi,
    canAutoSave: () =>
      configLoaded &&
      hotkeyCapture.isIdle &&
      !isOverlay &&
      !isToast &&
      hasTauriApi(),
    logFrontendError,
    onConfigSaved: (loaded) => {
      llmAutoAdapt.queueAfterSave(loaded.data);
    },
  });
  const windows = createWindowController({
    getConfig: () => config,
    applyLoadedConfig,
    safeInvoke,
    retryFailedSave: configController.retryFailedSave,
    discardUnsavedChanges: configController.discardUnsavedChanges,
    onHidden: clearSensitivePreviews,
  });
  const setup = createSetupController({
    t,
    safeInvoke,
    notify: notifications.show,
    getConfig: () => config,
    getStatusMessage: () => statusMessage,
    setStatusMessage: (message) => {
      statusMessage = message;
    },
    formatNumber,
    requireAuthFields,
    persistConfig,
    asrConfigFingerprint,
    localSetupStatusFromConfig,
    getSetupStatus: () => setupStatus,
    setSetupStatus: applySetupStatus,
    setSetupStatusLoading: (loading) => {
      setupStatusLoading = loading;
    },
    getAudioDevices: () => audioDevices,
    setAudioDevices: (devices) => {
      audioDevices = devices;
    },
    getTestingAsr: () => testingAsr,
    setTestingAsr: (testing) => {
      testingAsr = testing;
    },
    setAsrConnectionStatus: (status) => {
      asrConnectionStatus = status;
    },
    setAsrTestedConfigFingerprint: (fingerprint) => {
      asrTestedConfigFingerprint = fingerprint;
    },
    getTestingLlm: () => testingLlm,
    setTestingLlm: (testing) => {
      testingLlm = testing;
    },
    setLlmTestStatusMessage: (message) => {
      llmTestStatusMessage = message;
    },
    recordLlmTestResult: (record) => {
      llmTestHistory = appendLlmTestRecord(llmTestHistory, { ...record, testedAt: Date.now() });
      if (browser) saveLlmTestHistory(localStorage, llmTestHistory);
    },
    getTestingScreenContext: () => testingScreenContext,
    setTestingScreenContext: (testing) => {
      testingScreenContext = testing;
    },
    setScreenContextTestResult: (result) => {
      screenContextTestResult = result;
    },
  });
  llmAutoAdapt = createLlmAutoAdaptController({
    t,
    getConfig: () => config,
    isTesting: () => testingLlm,
    getTestStatusMessage: () => llmTestStatusMessage,
    setTestStatusMessage: (message) => {
      llmTestStatusMessage = message;
    },
    getTestedFingerprint: () => llmAutoAdaptTestedFingerprint,
    setTestedFingerprint: (fingerprint) => {
      llmAutoAdaptTestedFingerprint = fingerprint;
    },
    runTest: setup.testLlmConfig,
  });
  const session = createSessionController({
    t,
    safeInvoke,
    notify: notifications.show,
    requireAsrAuthGate,
    userErrorMessage,
    shouldOpenSettingsForError,
    scrollToSettingsPanel: settingsNav.scrollToSettingsPanel,
    settingsPanelForError,
    isConfigError,
    sessionPhaseMessage,
    scheduleSucceededIdleHint,
    getPhase: () => sessionPhase,
    setRecording: (value) => {
      recording = value;
    },
    setPhase: (value) => {
      if (startsNewRecordingSession(sessionPhase, value)) {
        notifications.advanceSessionRound();
      }
      sessionPhase = value;
    },
    setErrorCode: (value) => {
      sessionErrorCode = value ?? null;
    },
    setLastOutcome: (value) => {
      lastSessionOutcome = value;
    },
    setLastAudioQualityDiagnostic: (value) => {
      lastAudioQualityDiagnostic = value;
    },
    setAudioLevel: (value) => {
      audioLevel = value;
    },
    setStatusMessage: (value) => {
      statusMessage = value;
    },
    clearSensitivePreviews,
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
    applyDocumentMode();
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
      void overlay.refreshConfig(true);
      window.addEventListener("resize", overlay.refreshLayout);
      overlayPoll = window.setInterval(() => {
        void overlay.refreshText();
        void overlay.refreshConfig();
      }, 250);
    }
    let unlisteners: ReturnType<typeof registerNativeEventController> = [];
    if (hasTauriApi()) {
      unlisteners = registerNativeEventController({
        applySessionState,
        applyAsrFinalText: session.applyAsrFinalText,
        applyOverlayText: (payload) => {
          overlay.applyPayload(payload);
        },
        applyOverlayConfig: (payload) => {
          overlay.applyConfig(payload.ui);
        },
        applyStats: (payload) => {
          if (!isOverlay && !isToast) {
            stats.apply(payload);
            void autoHotwords.refreshStatus();
          }
        },
        applyAudioLevel: (payload) => {
          audioLevel = clampAudioLevel(payload.level);
        },
        applyAudioQuality: (payload) => {
          lastAudioQualityDiagnostic = payload;
        },
        handleAudioDeviceFallback: (payload) => {
          notifications.show(t("audioDeviceFallbackNotice", { device: payload.selected_name }), "warning");
          void refreshSetupStatus(false);
        },
        showClosePrompt: windows.showClosePrompt,
        showConfigExitGuard: windows.showSaveFailurePrompt,
        clearSensitivePreviews,
        checkForUpdate: () => {
          void updates.check(true);
        },
      });
      logFrontendEvent(`listeners registered mode=${frontendMode()}`);
    }
    return () => {
      if (overlayPoll !== undefined) window.clearInterval(overlayPoll);
      notifications.dispose();
      if (succeededIdleTimer !== undefined) window.clearTimeout(succeededIdleTimer);
      configController.dispose();
      overlay.dispose();
      clearDocumentMode();
      window.removeEventListener("resize", refreshMainDensity);
      window.removeEventListener("resize", overlay.refreshLayout);
      window.removeEventListener("error", onError);
      window.removeEventListener("unhandledrejection", onUnhandledRejection);
      disposeNativeEventController(unlisteners);
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

  $effect(() => {
    if (isOverlay || isToast || !hasTauriApi()) return;
    const active = shouldProtectUnsavedChanges(settingsDirty, configController.lastSaveError);
    void safeInvoke<void>("set_config_exit_guard", { active }, true);
  });

  function clearAutoSaveTimer() {
    configController.clearAutoSaveTimer();
  }
  function scheduleAutoSaveConfig() {
    configController.scheduleAutoSaveConfig();
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
      await maybeMigrateLegacyConfig();
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
  function applyDocumentMode() {
    const mode = frontendMode();
    document.documentElement.dataset.voxtypeMode = mode;
    document.body.dataset.voxtypeMode = mode;
  }
  function clearDocumentMode() {
    delete document.documentElement.dataset.voxtypeMode;
    delete document.body.dataset.voxtypeMode;
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
      const errorCode = invokeErrorCode(command);
      if (!quiet) {
        statusMessage = errorCode
          ? userErrorMessage(errorCode, "")
          : userFacingInvokeFailure(command, error, t("operationFailedGeneric"));
      }
      logFrontendError(`invoke failed command=${command}: ${formatFrontendError(error)}`);
      return null;
    }
  }

  function migrationDismissedKey(candidate: ConfigMigrationCandidate) {
    return `${configMigrationDismissedPrefix}${candidate.source_path}->${candidate.target_path}`;
  }

  async function maybeMigrateLegacyConfig() {
    if (!browser || isOverlay || isToast || !hasTauriApi()) return;
    const candidate = await safeInvoke<ConfigMigrationCandidate | null>(
      "get_config_migration_candidate",
      undefined,
      true,
    );
    if (!candidate) return;
    const dismissedKey = migrationDismissedKey(candidate);
    if (localStorage.getItem(dismissedKey) === "1") return;
    const confirmed = window.confirm(
      t("configMigrationConfirm", {
        source: candidate.source_path,
        target: candidate.target_path,
      }),
    );
    if (!confirmed) {
      localStorage.setItem(dismissedKey, "1");
      return;
    }
    const migrated = await safeInvoke<LoadedConfig>(
      "migrate_config_to_default_path",
      undefined,
      true,
    );
    if (migrated) {
      localStorage.removeItem(dismissedKey);
      notifications.show(t("configMigrationCompleted"), "success");
    } else {
      notifications.show(t("operationFailedGeneric"), "error");
    }
  }

  async function toggleRecordingFromUi() {
    await session.toggleRecordingFromUi(recording);
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
    return session.isBusy(sessionPhase);
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
    configController.applyLoadedConfig(loaded);
    llmAutoAdapt.markLoaded(loaded.data);
  }

  async function loadAll() {
    logFrontendEvent(`loadAll started mode=${frontendMode()}`);
    if (!isOverlay && !isToast && !setupStatus) setupStatusLoading = true;
    const [snapshotResult, configResult, statsResult, devicesResult, setupResult, localDataResult] = await Promise.all([
      safeInvoke<AppSnapshot>("get_app_snapshot"),
      loadAppConfig(),
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
    if (setupResult) {
      applySetupStatus(setupResult);
    } else if (!setupStatus && configResult) {
      setupStatus = localSetupStatusFromConfig(configResult.data, devicesResult ?? audioDevices);
    }
    if (!isOverlay && !isToast) setupStatusLoading = false;
    if (
      configLoadState !== "failed" &&
      (snapshotResult || configResult || statsResult) &&
      !configSetupMessage(configResult)
    ) {
      statusMessage = t("bridgeConnected");
    }
    logFrontendEvent(
      `loadAll completed mode=${frontendMode()} snapshot=${Boolean(snapshotResult)} config_loaded=${Boolean(configResult)} config_exists=${configResult?.exists ?? false} stats_records=${statsResult?.history.length ?? 0} audio_devices=${devicesResult?.length ?? 0} setup_ready=${setupResult?.ready ?? false} auto_hotword_entries=${autoHotwords.status?.entry_count ?? 0}`,
    );
    return loadedAny;
  }

  async function loadAppConfig() {
    if (!hasTauriApi()) {
      configLoadState = "missing";
      return null;
    }
    try {
      const loaded = await invoke<LoadedConfig>("load_app_config");
      configLoadState = configLoadStateForResult(loaded);
      return loaded;
    } catch (error) {
      configLoadState = "failed";
      statusMessage = t("configLoadFailed");
      logFrontendError(`load config failed: ${formatFrontendError(error)}`);
      if (!isOverlay && !isToast) {
        notifications.show(statusMessage, "error", {
          label: t("configLoadRetry"),
          onClick: retryLoadConfig,
        });
      }
      return null;
    }
  }

  async function retryLoadConfig() {
    const loaded = await loadAppConfig();
    if (!loaded) return;
    applyLoadedConfig(loaded);
    statusMessage = t("configLoadSucceeded");
    notifications.show(statusMessage, "success");
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
    const nextPhase = state.phase ?? (state.recording ? "recording" : "idle");
    if (nextPhase !== "succeeded" && succeededIdleTimer !== undefined) {
      window.clearTimeout(succeededIdleTimer);
      succeededIdleTimer = undefined;
    }
    session.applyState(state);
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
    return configController.persistConfig(options);
  }
  function fieldError(field: string) {
    return configController.fieldError(field);
  }
  function syncSetupStatusFromConfig(nextConfig: AppConfig) {
    const currentStatus = setupStatus ?? localSetupStatusFromConfig(nextConfig);
    applySetupStatus(mergeSetupStatusFromConfig(nextConfig, currentStatus));
  }
  function authFieldErrors() {
    const errors: Record<string, string> = {};
    if (isAliyunAsrProvider(config)) {
      if (!config.aliyun_asr.api_key.trim()) errors["aliyun_asr.api_key"] = t("requiredField");
      if (!config.aliyun_asr.model.trim()) errors["aliyun_asr.model"] = t("requiredField");
      if (!config.aliyun_asr.workspace_id.trim() && !config.aliyun_asr.websocket_url.trim()) {
        errors["aliyun_asr.workspace_id"] = t("aliyunWorkspaceOrUrlRequired");
      }
    } else {
      if (config.auth.mode === DOUBAO_AUTH_MODE_AGENT_PLAN) {
        if (!config.auth.api_key.trim()) errors["auth.api_key"] = t("requiredField");
      } else {
        if (!config.auth.app_key.trim()) errors["auth.app_key"] = t("requiredField");
        if (!config.auth.access_key.trim()) errors["auth.access_key"] = t("requiredField");
      }
    }
    return errors;
  }
  function clearAuthFieldErrors() {
    const next = { ...validationErrors };
    delete next["auth.app_key"];
    delete next["auth.access_key"];
    delete next["auth.api_key"];
    delete next["auth.mode"];
    delete next["auth.resource_id"];
    delete next["asr.provider"];
    delete next["aliyun_asr.api_key"];
    delete next["aliyun_asr.workspace_id"];
    delete next["aliyun_asr.websocket_url"];
    delete next["aliyun_asr.region"];
    delete next["aliyun_asr.model"];
    delete next["aliyun_asr.language_hint"];
    delete next["aliyun_asr.max_sentence_silence"];
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
    await setup.testAsrConfig();
  }
  async function testLlmConfig() {
    await llmAutoAdapt.testManually();
  }
  async function testScreenContext() {
    await setup.testScreenContext();
  }
  function clearSensitivePreviews() {
    lastSessionOutcome = null;
    screenContextTestResult = null;
  }
  function clearLastOutcome() {
    lastSessionOutcome = null;
  }
  function clearScreenContextPreview() {
    screenContextTestResult = null;
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
    const configuredName = config.audio.input_device_name?.trim();
    if (config.audio.input_device !== null && config.audio.input_device !== undefined) {
      const configured = audioDevices.find((device) => device.index === config.audio.input_device);
      if (configured && (!configuredName || configured.name.trim().toLowerCase() === configuredName.toLowerCase())) {
        return configured;
      }
    }
    if (configuredName) {
      const configured = audioDevices.filter((device) => device.name.trim().toLowerCase() === configuredName.toLowerCase());
      if (configured.length === 1) return configured[0];
      return audioDevices.find((device) => device.is_default) ?? audioDevices[0];
    }
    return audioDevices.find((device) => device.is_default) ?? audioDevices[0];
  }
  async function refreshSetupStatus(showLoading = true) {
    await setup.refreshSetupStatus(showLoading);
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
  function openRecordingTroubleshooting() {
    settingsNav.scrollToSettingsPanel("settings-recording-troubleshooting");
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
      config.audio.input_device_name = null;
      return;
    }
    const index = Number(value);
    const device = audioDevices.find((item) => item.index === index);
    config.audio.input_device = index;
    config.audio.input_device_name = device?.name ?? null;
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
  function configSaveState() {
    return configController.configSaveState(isOverlay, isToast);
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
  function llmTestStatusText() {
    return llmAutoAdapt.statusText();
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
      configSaveError: configController.lastSaveError,
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
      onHideToTray: windows.hideToTrayFromTitlebar,
      onRequestClose: windows.requestClose,
      onRetrySave: () => {
        void configController.retryFailedSave();
      },
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
      saving: configController.saving,
      inputStatus: inputStatus(),
      inputStatusLabel: inputStatusLabel(),
      inputStatusDesc: inputStatusDesc(),
      requiresAsrAuth: requiresAsrAuth(),
      setupRequiredMessage,
      activeErrorDetail: activeUserErrorDetail(),
      activeErrorActions: activeUserErrorActions(),
      lastSessionOutcome,
      lastAudioQualityDiagnostic,
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
      llmTestStatusText: llmTestStatusText(),
      llmTestSummary: summarizeLlmTestHistory(llmTestHistory),
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
      onOpenRecordingTroubleshooting: openRecordingTroubleshooting,
      onUserErrorAction: handleUserErrorAction,
      onCopyLastOutcomeText: copyLastOutcomeText,
      onClearLastOutcome: clearLastOutcome,
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
      onOpenAliyunAsrDocs: openAliyunAsrDocs,
      onTestAsrConfig: testAsrConfig,
      onTestLlmConfig: testLlmConfig,
      onTestScreenContext: testScreenContext,
      onClearScreenContextPreview: clearScreenContextPreview,
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
      onOpenRecentContextSettings: () => settingsNav.scrollToSettingsPanel("settings-prompt-context"),
      onOpenAutoHotwordSettings: () => settingsNav.scrollToSettingsPanel("settings-auto-hotwords"),
      onOpenScreenContextSettings: () => settingsNav.scrollToSettingsPanel("settings-screen-context"),
      onOpenOutputSettings: () => settingsNav.scrollToSettingsPanel("settings-basic-output"),
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
    await safeInvoke<void>("open_doubao_asr_docs", { mode: config.auth.mode });
  }
  async function openAliyunAsrDocs() {
    await safeInvoke<void>("open_aliyun_asr_docs");
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
    get actionNoticeCloseLabel() { return t("noticeClose"); },
    get closePromptVisible() { return windows.closePromptVisible; },
    get saveFailurePromptVisible() { return windows.saveFailurePromptVisible; },
    get saveFailurePromptTitle() { return t("saveFailurePromptTitle"); },
    get saveFailurePromptBody() { return t("saveFailurePromptBody"); },
    get saveFailurePromptError() { return configController.lastSaveError; },
    get saveFailureRetryLabel() { return configController.saving ? t("settingsSaving") : t("saveFailureRetry"); },
    get saveFailureDiscardLabel() { return t("saveFailureDiscard"); },
    get saveFailureCancelLabel() { return t("saveFailureCancel"); },
    get configEditable() { return canEditLoadedConfig(configLoadState); },
    get savingConfig() { return configController.saving; },
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
    closeActionNotice: notifications.clear,
    overlayMeterBarHeight: overlay.meterBarHeight,
    overlayMeterBarOpacity: overlay.meterBarOpacity,
    closeWindowWithoutFuturePrompt: windows.closeWithoutFuturePrompt,
    exitFromClosePrompt: windows.exitFromPrompt,
    confirmClosePrompt: windows.confirmClosePrompt,
    retrySaveAndContinue: windows.retrySaveAndContinue,
    discardAndContinue: windows.discardAndContinue,
    cancelSaveFailurePrompt: windows.cancelSaveFailurePrompt,
    closePromptPreview,
    copyPromptPreview,
  };
}
