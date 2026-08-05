import type { AppConfig } from "$lib/types/app";

export const ASR_PROVIDER_DOUBAO = "doubao";
export const ASR_PROVIDER_ALIYUN_FUN = "aliyun_fun";
export const DOUBAO_AUTH_MODE_APP_ACCESS = "app_access";
export const DOUBAO_AUTH_MODE_AGENT_PLAN = "agent_plan";
export const DOUBAO_SEED_ASR_2_RESOURCE_ID = "volc.seedasr.sauc.duration";

// Frontend mirror of the Rust provider facade. Keep this file focused on
// provider selection and UI readiness checks; protocol validation stays in Rust.
export function activeAsrProvider(config: AppConfig) {
  return config.asr.provider === ASR_PROVIDER_ALIYUN_FUN ? ASR_PROVIDER_ALIYUN_FUN : ASR_PROVIDER_DOUBAO;
}

export function isAliyunAsrProvider(config: AppConfig) {
  return activeAsrProvider(config) === ASR_PROVIDER_ALIYUN_FUN;
}

export function hasAsrProviderConfig(config: AppConfig) {
  if (isAliyunAsrProvider(config)) {
    return Boolean(
      config.aliyun_asr.api_key.trim() &&
        config.aliyun_asr.model.trim() &&
        (config.aliyun_asr.workspace_id.trim() || config.aliyun_asr.websocket_url.trim()),
    );
  }
  if (config.auth.mode === DOUBAO_AUTH_MODE_AGENT_PLAN) {
    return Boolean(
      config.auth.api_key.trim() &&
        config.auth.resource_id.trim() === DOUBAO_SEED_ASR_2_RESOURCE_ID,
    );
  }
  return Boolean(
    config.auth.app_key.trim() && config.auth.access_key.trim() && config.auth.resource_id.trim(),
  );
}

// Used to refresh setup status when the active provider's effective inputs change.
// Do not include recognition text, prompt bodies, recent context, or screen OCR here.
export function activeAsrConfigFingerprint(config: AppConfig) {
  if (isAliyunAsrProvider(config)) {
    return JSON.stringify({
      provider: ASR_PROVIDER_ALIYUN_FUN,
      api_key: config.aliyun_asr.api_key,
      workspace_id: config.aliyun_asr.workspace_id,
      websocket_url: config.aliyun_asr.websocket_url,
      region: config.aliyun_asr.region,
      model: config.aliyun_asr.model,
      language_hint: config.aliyun_asr.language_hint,
      semantic_punctuation_enabled: config.aliyun_asr.semantic_punctuation_enabled,
      max_sentence_silence: config.aliyun_asr.max_sentence_silence,
      vocabulary_id: config.aliyun_asr.vocabulary_id,
    });
  }
  return JSON.stringify({
    provider: ASR_PROVIDER_DOUBAO,
    mode: config.auth.mode,
    app_key: config.auth.app_key,
    access_key: config.auth.access_key,
    api_key: config.auth.api_key,
    resource_id: config.auth.resource_id,
    ws_url: config.request.ws_url,
    model_name: config.request.model_name,
    language: config.request.language,
  });
}
