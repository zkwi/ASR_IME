import { fallbackConfig, fallbackSnapshot, setupStatusCacheKey } from "$lib/app/defaults";
import type { SetupStatusItem, SetupStatusWarning } from "$lib/components/overview/SetupStatusCard.svelte";
import type { AppConfig, AsrConnectionStatus, AudioDeviceInfo } from "$lib/types/app";
import type { CopyKey } from "$lib/i18n";
import { activeAsrConfigFingerprint, hasAsrProviderConfig } from "$lib/utils/asrProvider";

export type SetupStatus = {
  ready: boolean;
  missing_auth: boolean;
  has_audio_device: boolean;
  hotkey: string;
  paste_method: string;
  privacy_recent_context_enabled: boolean;
  warnings: SetupStatusWarning[];
};

type Translate = (key: CopyKey, values?: Record<string, string>) => string;

export function readCachedSetupStatus(isBrowser: boolean): SetupStatus | null {
  if (!isBrowser) return null;
  const params = new URLSearchParams(window.location.search);
  if (params.has("overlay") || params.has("toast")) return null;
  try {
    const raw = localStorage.getItem(setupStatusCacheKey);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<SetupStatus>;
    if (
      typeof parsed.ready !== "boolean" ||
      typeof parsed.missing_auth !== "boolean" ||
      typeof parsed.has_audio_device !== "boolean"
    ) {
      return null;
    }
    return {
      ready: parsed.ready,
      missing_auth: parsed.missing_auth,
      has_audio_device: parsed.has_audio_device,
      hotkey: typeof parsed.hotkey === "string" ? parsed.hotkey : fallbackSnapshot.hotkey,
      paste_method: typeof parsed.paste_method === "string" ? parsed.paste_method : fallbackConfig.typing.paste_method,
      privacy_recent_context_enabled: Boolean(parsed.privacy_recent_context_enabled),
      warnings: Array.isArray(parsed.warnings) ? parsed.warnings : [],
    };
  } catch {
    return null;
  }
  return null;
}

export function buildLocalSetupStatus(config: AppConfig, devices: AudioDeviceInfo[] = [], warnings: SetupStatusWarning[] = []): SetupStatus {
  const missingAuth = !hasAsrProviderConfig(config);
  const anyTriggerEnabled =
    config.triggers.hotkey_enabled ||
    config.triggers.middle_mouse_enabled ||
    config.triggers.right_alt_enabled;
  return {
    ready: !missingAuth && devices.length > 0 && anyTriggerEnabled,
    missing_auth: missingAuth,
    has_audio_device: devices.length > 0,
    hotkey: config.hotkey,
    paste_method: config.typing.paste_method,
    privacy_recent_context_enabled: config.context.enable_recent_context,
    warnings,
  };
}

export function mergeSetupStatusFromConfig(config: AppConfig, currentStatus: SetupStatus): SetupStatus {
  const missingAuth = !hasAsrProviderConfig(config);
  const anyTriggerEnabled =
    config.triggers.hotkey_enabled ||
    config.triggers.middle_mouse_enabled ||
    config.triggers.right_alt_enabled;
  return {
    ...currentStatus,
    missing_auth: missingAuth,
    hotkey: config.hotkey,
    paste_method: config.typing.paste_method,
    privacy_recent_context_enabled: config.context.enable_recent_context,
    ready: !missingAuth && currentStatus.has_audio_device && anyTriggerEnabled,
  };
}

export function pasteMethodLabel(value: string, t: Translate) {
  if (value === "clipboard_only") return t("clipboardOnly");
  if (value === "shift_insert") return "Shift + Insert";
  return "Ctrl + V";
}

export function asrConfigFingerprint(config: AppConfig) {
  return activeAsrConfigFingerprint(config);
}

export function currentAsrConnectionStatus(params: {
  status: SetupStatus | null;
  authReady: boolean;
  testingAsr: boolean;
  currentFingerprint: string;
  testedFingerprint: string;
  asrConnectionStatus: AsrConnectionStatus;
}): AsrConnectionStatus {
  const authReady = params.status ? !params.status.missing_auth : params.authReady;
  if (!authReady) return "missing_auth";
  if (params.testingAsr) return "testing";
  if (
    params.testedFingerprint === params.currentFingerprint &&
    (params.asrConnectionStatus === "tested_ok" || params.asrConnectionStatus === "tested_failed")
  ) {
    return params.asrConnectionStatus;
  }
  return "configured_not_tested";
}

export function asrConnectionStatusText(status: AsrConnectionStatus, t: Translate) {
  if (status === "missing_auth") return t("setupAsrMissingAuth");
  if (status === "testing") return t("setupAsrTesting");
  if (status === "tested_ok") return t("setupAsrTestedOk");
  if (status === "tested_failed") return t("setupAsrTestedFailed");
  return t("setupAsrConfiguredNotTested");
}

export function asrConnectionStatusOk(status: AsrConnectionStatus) {
  return status === "configured_not_tested" || status === "tested_ok";
}

export function formatEnabledTriggers(config: AppConfig, hotkey: string, t: Translate, formatHotkey: (value: string) => string) {
  const triggers = [];
  if (config.triggers.hotkey_enabled) triggers.push(formatHotkey(hotkey));
  if (config.triggers.middle_mouse_enabled) triggers.push(t("middleMouse"));
  if (config.triggers.right_alt_enabled) triggers.push(t("rightAlt"));
  return triggers.length > 0 ? triggers.join(" / ") : t("disabled");
}

export function buildSetupStatusItems(params: {
  loading: boolean;
  configLoaded: boolean;
  config: AppConfig;
  setupStatus: SetupStatus | null;
  localStatus: SetupStatus;
  audioDevices: AudioDeviceInfo[];
  asrStatus: AsrConnectionStatus;
  triggerText: string;
  t: Translate;
}): SetupStatusItem[] {
  if (params.loading && !params.setupStatus) {
    return [
      { label: params.t("setupAuthLabel"), value: params.t("setupChecking"), ok: false, checking: true, action: "asr_auth" },
      { label: params.t("setupMicLabel"), value: params.t("setupChecking"), ok: false, checking: true, action: "audio" },
      { label: params.t("setupPasteLabel"), value: params.t("setupChecking"), ok: false, checking: true, action: "typing" },
      { label: params.t("setupTriggerLabel"), value: params.t("setupChecking"), ok: false, checking: true, action: "hotkey" },
      { label: params.t("setupPrivacyLabel"), value: params.t("setupChecking"), ok: false, checking: true, action: "privacy" },
    ];
  }
  const status = params.setupStatus ?? params.localStatus;
  const micReady = status ? status.has_audio_device : params.audioDevices.length > 0;
  return [
    {
      label: params.t("setupAuthLabel"),
      value: asrConnectionStatusText(params.asrStatus, params.t),
      ok: asrConnectionStatusOk(params.asrStatus),
      action: "asr_auth",
    },
    {
      label: params.t("setupMicLabel"),
      value: micReady ? params.t("setupMicDetected") : params.t("setupMicMissing"),
      ok: micReady,
      action: "audio",
    },
    {
      label: params.t("setupPasteLabel"),
      value: pasteMethodLabel(params.configLoaded ? params.config.typing.paste_method : status.paste_method, params.t),
      ok: true,
      action: "typing",
    },
    {
      label: params.t("setupTriggerLabel"),
      value: params.triggerText,
      ok: params.config.triggers.hotkey_enabled || params.config.triggers.middle_mouse_enabled || params.config.triggers.right_alt_enabled,
      action: "hotkey",
    },
    {
      label: params.t("setupPrivacyLabel"),
      value: params.t("setupPrivacyChecked"),
      ok: true,
      action: "privacy",
    },
  ];
}

const setupWarningCopyKeys: Record<string, { title: CopyKey; message: CopyKey }> = {
  ASR_AUTH_MISSING: {
    title: "setupWarningAsrAuthTitle",
    message: "setupWarningAsrAuthMessage",
  },
  MIC_DEVICE_NOT_FOUND: {
    title: "setupWarningMicTitle",
    message: "setupWarningMicMessage",
  },
  TRIGGER_DISABLED: {
    title: "setupWarningTriggerTitle",
    message: "setupWarningTriggerMessage",
  },
};

export function localizeSetupWarnings(warnings: SetupStatusWarning[], t: Translate): SetupStatusWarning[] {
  return warnings.map((warning) => {
    const copyKeys = setupWarningCopyKeys[warning.code];
    if (!copyKeys) return warning;
    return {
      ...warning,
      title: t(copyKeys.title),
      message: t(copyKeys.message),
    };
  });
}

export function setupActionText(action: string, t: Translate) {
  if (action === "asr_auth") return t("setupActionAsr");
  if (action === "audio") return t("setupActionAudio");
  if (action === "typing") return t("setupActionTyping");
  if (action === "hotkey") return t("setupActionHotkey");
  if (action === "privacy") return t("setupActionPrivacy");
  return t("setupCta");
}
