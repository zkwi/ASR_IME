import type { AppConfig } from "$lib/types/app";

export const LLM_THINKING_STRATEGY_AUTO = "auto";

export function hasLlmAdapterTestConfig(config: AppConfig) {
  const llm = config.llm_post_edit;
  return Boolean(llm.base_url?.trim() && llm.api_key?.trim() && llm.model?.trim());
}

export function llmAdapterConfigFingerprint(config: AppConfig) {
  const llm = config.llm_post_edit;
  return JSON.stringify({
    base_url: llm.base_url.trim().replace(/\/+$/, ""),
    api_key: llm.api_key.trim(),
    model: llm.model.trim(),
    enable_thinking: llm.enable_thinking,
  });
}
