import type { CopyKey, Language, UserErrorDetail } from "$lib/i18n";
import { userErrorDetails } from "$lib/i18n";
import type { AppConfig, LoadedConfig, Section } from "$lib/types/app";
import type { SetupStatus } from "$lib/utils/setupStatus";

type Translate = (key: CopyKey, values?: Record<string, string>) => string;

export function hasAuth(config: AppConfig) {
  return Boolean(config.auth.app_key.trim() && config.auth.access_key.trim());
}

export function hasLlmApiConfig(config: AppConfig) {
  return Boolean(
    config.llm_post_edit.base_url?.trim() &&
      config.llm_post_edit.api_key?.trim() &&
      config.llm_post_edit.model?.trim(),
  );
}

export function requiresAsrAuth(params: {
  configLoaded: boolean;
  setupStatus: SetupStatus | null;
  config: AppConfig;
  configExists: boolean;
  targetConfig?: AppConfig;
  targetExists?: boolean;
}) {
  if (params.targetConfig === undefined && params.targetExists === undefined && !params.configLoaded) {
    return params.setupStatus ? params.setupStatus.missing_auth : false;
  }
  const config = params.targetConfig ?? params.config;
  const exists = params.targetExists ?? params.configExists;
  return !exists || !hasAuth(config);
}

export function configSetupMessage(loaded: LoadedConfig | null, t: Translate) {
  if (!loaded) return "";
  if (!loaded.exists) return t("setupMissingFile");
  if (!hasAuth(loaded.data)) return t("setupMissingAuth");
  return "";
}

export function sectionForSettingsPanel(targetId: string): Section {
  if (targetId === "settings-context" || targetId === "settings-prompt-context" || targetId === "settings-llm-prompt" || targetId === "settings-auto-hotwords") {
    return "Hotwords";
  }
  if (targetId === "settings-auth" || targetId === "settings-asr-language" || targetId === "settings-llm-api") {
    return "ApiConfig";
  }
  return "Options";
}

export function isConfigError(message: string) {
  return (
    message.includes("ASR 未配置") ||
    message.includes("config.toml") ||
    message.includes("app_key") ||
    message.includes("access_key") ||
    message.includes("豆包 ASR 认证") ||
    message.includes("豆包 ASR Key") ||
    message.includes("Doubao ASR") ||
    message.includes("App Key") ||
    message.includes("Access Key") ||
    message.includes("Resource ID")
  );
}

export function userErrorDetail(
  code: string | null | undefined,
  fallback: string,
  language: Language,
  t: Translate,
): UserErrorDetail {
  const matchedDetail = code ? userErrorDetails[language][code] : undefined;
  if (matchedDetail) return matchedDetail;
  if (isConfigError(fallback)) return userErrorDetails[language].ASR_AUTH_MISSING;
  if (fallback.includes("剪贴板") || fallback.toLowerCase().includes("clipboard")) {
    return userErrorDetails[language].CLIPBOARD_WRITE_FAILED;
  }
  if (fallback.includes("麦克风") || fallback.toLowerCase().includes("microphone")) {
    return userErrorDetails[language].MIC_START_FAILED;
  }
  return {
    title: t("inputError"),
    cause: t("genericErrorCause"),
    action: t("genericErrorAction"),
  };
}

export function userErrorMessage(
  code: string | null | undefined,
  fallback: string,
  language: Language,
  t: Translate,
) {
  const detail = userErrorDetail(code, fallback, language, t);
  const separator = language === "en" ? ". " : "。";
  return `${detail.title}${separator}${detail.action}`;
}

export function isErrorStatus(message: string) {
  return (
    isConfigError(message) ||
    message.includes("无法连接豆包 ASR") ||
    message.includes("连接豆包 ASR 失败") ||
    message.includes("豆包 ASR 服务返回错误码") ||
    message.includes("开机自启动设置失败") ||
    message.includes("启动录音失败") ||
    message.includes("麦克风采集异常")
  );
}

export function shouldOpenSettingsForError(message: string, code?: string | null) {
  return (
    code === "CONFIG_MISSING" ||
    code === "ASR_AUTH_MISSING" ||
    code === "MIC_DEVICE_NOT_FOUND" ||
    code === "MIC_STREAM_FAILED" ||
    isConfigError(message) ||
    message.includes("API Key") ||
    message.includes("Base URL")
  );
}

export function settingsPanelForError(message: string, code?: string | null) {
  if (
    code === "MIC_DEVICE_NOT_FOUND" ||
    code === "MIC_STREAM_FAILED" ||
    message.includes("麦克风") ||
    message.toLowerCase().includes("microphone")
  ) {
    return "settings-audio";
  }
  return "settings-auth";
}
