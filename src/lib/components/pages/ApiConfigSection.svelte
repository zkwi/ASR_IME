<script lang="ts">
  import AdvancedSettings from "$lib/components/common/AdvancedSettings.svelte";
  import SettingTags from "$lib/components/common/SettingTags.svelte";
  import SetupStatusCard, {
    type SetupStatusItem,
    type SetupStatusWarning,
  } from "$lib/components/overview/SetupStatusCard.svelte";
  import type { CopyKey } from "$lib/i18n";
  import type { AppConfig } from "$lib/types/app";
  import { ASR_PROVIDER_ALIYUN_FUN, ASR_PROVIDER_DOUBAO } from "$lib/utils/asrProvider";
  import { ExternalLink, ShieldCheck } from "lucide-svelte";

  type Translate = (key: CopyKey, values?: Record<string, string>) => string;

  const asrLanguageOptions: Array<{ value: string; labelKey: CopyKey }> = [
    { value: "", labelKey: "asrLanguageAuto" },
    { value: "en-US", labelKey: "asrLanguageEnUS" },
    { value: "ja-JP", labelKey: "asrLanguageJaJP" },
    { value: "id-ID", labelKey: "asrLanguageIdID" },
    { value: "es-MX", labelKey: "asrLanguageEsMX" },
    { value: "pt-BR", labelKey: "asrLanguagePtBR" },
    { value: "de-DE", labelKey: "asrLanguageDeDE" },
    { value: "fr-FR", labelKey: "asrLanguageFrFR" },
    { value: "ko-KR", labelKey: "asrLanguageKoKR" },
    { value: "fil-PH", labelKey: "asrLanguageFilPH" },
    { value: "ms-MY", labelKey: "asrLanguageMsMY" },
    { value: "th-TH", labelKey: "asrLanguageThTH" },
    { value: "ar-SA", labelKey: "asrLanguageArSA" },
    { value: "it-IT", labelKey: "asrLanguageItIT" },
    { value: "bn-BD", labelKey: "asrLanguageBnBD" },
    { value: "el-GR", labelKey: "asrLanguageElGR" },
    { value: "nl-NL", labelKey: "asrLanguageNlNL" },
    { value: "ru-RU", labelKey: "asrLanguageRuRU" },
    { value: "tr-TR", labelKey: "asrLanguageTrTR" },
    { value: "vi-VN", labelKey: "asrLanguageViVN" },
    { value: "pl-PL", labelKey: "asrLanguagePlPL" },
    { value: "ro-RO", labelKey: "asrLanguageRoRO" },
    { value: "ne-NP", labelKey: "asrLanguageNeNP" },
    { value: "uk-UA", labelKey: "asrLanguageUkUA" },
    { value: "yue-CN", labelKey: "asrLanguageYueCN" },
  ];

  const llmThinkingStrategyOptions: Array<{ value: string; labelKey: CopyKey }> = [
    { value: "auto", labelKey: "llmThinkingStrategyAuto" },
    { value: "dashscope_enable_thinking", labelKey: "llmThinkingStrategyDashscope" },
    { value: "thinking_disabled", labelKey: "llmThinkingStrategyDisabledObject" },
    { value: "openrouter_reasoning_low", labelKey: "llmThinkingStrategyOpenRouterLow" },
    { value: "openrouter_reasoning_minimal", labelKey: "llmThinkingStrategyOpenRouterMinimal" },
    { value: "omit", labelKey: "llmThinkingStrategyOmit" },
  ];

  const aliyunLanguageOptions: Array<{ value: string; labelKey: CopyKey }> = [
    { value: "", labelKey: "aliyunLanguageAuto" },
    { value: "zh", labelKey: "aliyunLanguageZh" },
    { value: "en", labelKey: "aliyunLanguageEn" },
    { value: "ja", labelKey: "aliyunLanguageJa" },
    { value: "ko", labelKey: "aliyunLanguageKo" },
    { value: "yue", labelKey: "aliyunLanguageYue" },
  ];

  const aliyunRegionOptions: Array<{ value: string; labelKey: CopyKey }> = [
    { value: "cn-beijing", labelKey: "aliyunRegionCnBeijing" },
    { value: "ap-southeast-1", labelKey: "aliyunRegionSingapore" },
  ];

  type Props = {
    config: AppConfig;
    t: Translate;
    configExists: boolean;
    setupChecking: boolean;
    setupStatusReady: boolean;
    setupStatusItems: SetupStatusItem[];
    setupWarnings: SetupStatusWarning[];
    setupWarningCount: number;
    snapshotHotkey: string;
    requiresAsrAuth: boolean;
    testingAsr: boolean;
    testingLlm: boolean;
    hasLlmApiConfig: boolean;
    llmApiStatusText: string;
    llmTestStatusText: string;
    fieldError: (field: string) => string;
    setupRequiredMessage: () => string;
    setupActionText: (action: string) => string;
    formatHotkey: (value: string) => string;
    onScrollToSettingsPanel: (id: string) => void;
    onOpenSetupGuide: () => void;
    onOpenDoubaoAsrDocs: () => void;
    onOpenAliyunAsrDocs: () => void;
    onRefreshSetupStatus: () => void;
    onSetupAction: (action: string) => void;
    onTestAsrConfig: () => void;
    onTestLlmConfig: () => void;
  };

  let {
    config = $bindable<AppConfig>(),
    t,
    configExists,
    setupChecking,
    setupStatusReady,
    setupStatusItems,
    setupWarnings,
    setupWarningCount,
    snapshotHotkey,
    requiresAsrAuth,
    testingAsr,
    testingLlm,
    hasLlmApiConfig,
    llmApiStatusText,
    llmTestStatusText,
    fieldError,
    setupRequiredMessage,
    setupActionText,
    formatHotkey,
    onScrollToSettingsPanel,
    onOpenSetupGuide,
    onOpenDoubaoAsrDocs,
    onOpenAliyunAsrDocs,
    onRefreshSetupStatus,
    onSetupAction,
    onTestAsrConfig,
    onTestLlmConfig,
  }: Props = $props();

  let showAsrAdvanced = $state(false);
  let showLlmAdvanced = $state(false);
  let asrAdvancedHasError = $derived(Boolean(
    fieldError("aliyun_asr.region") ||
      fieldError("aliyun_asr.model") ||
      fieldError("aliyun_asr.websocket_url") ||
      fieldError("aliyun_asr.language_hint") ||
      fieldError("request.language"),
  ));
  let llmAdvancedHasError = $derived(Boolean(fieldError("llm_post_edit.thinking_strategy")));
  let asrAdvancedExpanded = $derived(showAsrAdvanced || asrAdvancedHasError);
  let llmAdvancedExpanded = $derived(showLlmAdvanced || llmAdvancedHasError);
</script>

<section class="settings-stack">
  {#if requiresAsrAuth}
    <section class="auth-gate-card" aria-live="polite">
      <div class="auth-gate-copy">
        <strong>{t("apiOnboardingTitle")}</strong>
        <p>{!configExists ? t("setupMissingFile") : t("apiOnboardingDescription")}</p>
        <ol class="starter-steps" aria-label={t("apiOnboardingStepsLabel")}>
          <li>
            <span>1</span>
            <div>
              <strong>{t("apiOnboardingStepKeysTitle")}</strong>
              <small>{t("apiOnboardingStepKeysDescription")}</small>
            </div>
          </li>
          <li>
            <span>2</span>
            <div>
              <strong>{t("apiOnboardingStepTestTitle")}</strong>
              <small>{t("apiOnboardingStepTestDescription")}</small>
            </div>
          </li>
          <li>
            <span>3</span>
            <div>
              <strong>{t("apiOnboardingStepStartTitle")}</strong>
              <small>{t("apiOnboardingStepStartDescription", { hotkey: formatHotkey(snapshotHotkey) })}</small>
            </div>
          </li>
        </ol>
      </div>
      <div class="setup-actions">
        <button type="button" onclick={() => onScrollToSettingsPanel("settings-auth")}>{t("apiOnboardingPrimaryCta")}</button>
        <button type="button" class="secondary" onclick={onOpenSetupGuide}>{t("setupGuideCta")}</button>
      </div>
    </section>
  {/if}
  <div class="settings-group-heading">
    <h3>{t("apiConfigPageTitle")}</h3>
    <p>{t("apiConfigPageDescription")}</p>
  </div>
  <SetupStatusCard
    ready={setupStatusReady}
    checking={setupChecking}
    items={setupStatusItems}
    warnings={setupWarnings}
    texts={{
      title: t("setupHealthTitle"),
      pendingTitle: t("setupHealthPendingTitle", { count: String(setupWarningCount) }),
      pendingDescription: t("setupHealthPendingDescription"),
      checkingTitle: t("setupHealthCheckingTitle"),
      checkingDescription: t("setupHealthCheckingDescription"),
      readyTitle: t("setupHealthReadyTitle"),
      readyDescription: t("setupHealthReadyDescription", { hotkey: formatHotkey(snapshotHotkey) }),
      refresh: t("refreshSetup"),
      warningSummary: (count: number) => t("setupWarningSummary", { count: String(count) }),
      actionText: setupActionText,
    }}
    onAction={onSetupAction}
    onRefresh={onRefreshSetupStatus}
  />
  <section class="settings-group">
    <div class="settings-group-heading">
      <h3>{t("speechRecognitionApiTitle")}</h3>
      <p>{t("speechRecognitionApiDescription")}</p>
    </div>
    <div id="settings-auth" class="form-panel">
      <div class="section-heading with-actions">
        <div class="section-heading-copy">
          <h3>{t("asrProviderAuthTitle")}</h3>
          <p>{config.asr.provider === ASR_PROVIDER_ALIYUN_FUN ? t("aliyunAuthRequiredHint") : t("doubaoAuthRequiredHint")}</p>
          <SettingTags tags={[{ label: t("tagRequired"), tone: "required" }, t("tagSentToService")]} />
          {#if requiresAsrAuth}
            <p class="setup-note">{setupRequiredMessage()}</p>
            <button class="link-button" type="button" onclick={onOpenSetupGuide}>{t("setupGuideCta")}</button>
          {/if}
        </div>
        <div class="settings-inline-actions">
          <button
            class="link-button"
            type="button"
            onclick={config.asr.provider === ASR_PROVIDER_ALIYUN_FUN ? onOpenAliyunAsrDocs : onOpenDoubaoAsrDocs}
          >
            <ExternalLink size={16} />{config.asr.provider === ASR_PROVIDER_ALIYUN_FUN ? t("aliyunDocsCta") : t("doubaoDocsCta")}
          </button>
          <button class="test-button" type="button" onclick={onTestAsrConfig} disabled={testingAsr}>
            <ShieldCheck size={16} />{testingAsr ? t("testingConnection") : t("testConnection")}
          </button>
        </div>
      </div>
      <div class="form-grid">
        <label class:field-invalid={Boolean(fieldError("asr.provider"))}>
          <span>{t("asrProviderSelect")}</span>
          <select bind:value={config.asr.provider}>
            <option value={ASR_PROVIDER_DOUBAO}>{t("asrProviderDoubao")}</option>
            <option value={ASR_PROVIDER_ALIYUN_FUN}>{t("asrProviderAliyun")}</option>
          </select>
          {#if fieldError("asr.provider")}<small class="field-error">{fieldError("asr.provider")}</small>{/if}
          <small class="field-hint">{t("asrProviderHint")}</small>
        </label>
        {#if config.asr.provider === ASR_PROVIDER_ALIYUN_FUN}
          <label class:field-invalid={Boolean(fieldError("aliyun_asr.api_key"))}>
            <span>API Key</span>
            <input type="password" autocomplete="off" bind:value={config.aliyun_asr.api_key} />
            {#if fieldError("aliyun_asr.api_key")}<small class="field-error">{fieldError("aliyun_asr.api_key")}</small>{/if}
            <small class="field-hint">{t("aliyunApiKeyHint")}</small>
          </label>
          <label class:field-invalid={Boolean(fieldError("aliyun_asr.workspace_id"))}>
            <span>Workspace ID</span>
            <input autocomplete="off" bind:value={config.aliyun_asr.workspace_id} />
            {#if fieldError("aliyun_asr.workspace_id")}<small class="field-error">{fieldError("aliyun_asr.workspace_id")}</small>{/if}
            <small class="field-hint">{t("aliyunWorkspaceHint")}</small>
          </label>
        {:else}
          <label class:field-invalid={Boolean(fieldError("auth.app_key"))}>
            <span>{t("appKey")}</span>
            <input autocomplete="off" bind:value={config.auth.app_key} />
            {#if fieldError("auth.app_key")}<small class="field-error">{fieldError("auth.app_key")}</small>{/if}
          </label>
          <label class:field-invalid={Boolean(fieldError("auth.access_key"))}>
            <span>{t("accessKey")}</span>
            <input type="password" autocomplete="off" bind:value={config.auth.access_key} />
            {#if fieldError("auth.access_key")}<small class="field-error">{fieldError("auth.access_key")}</small>{/if}
          </label>
        {/if}
      </div>
      <AdvancedSettings
        title={t("asrAdvancedSettings")}
        description={t("asrAdvancedSettingsHint")}
        expanded={asrAdvancedExpanded}
        panelId="settings-asr-language"
        onToggle={() => (showAsrAdvanced = !showAsrAdvanced)}
      >
        <div class="form-grid">
          {#if config.asr.provider === ASR_PROVIDER_ALIYUN_FUN}
            <label class:field-invalid={Boolean(fieldError("aliyun_asr.region"))}>
              <span>{t("aliyunRegion")}</span>
              <select bind:value={config.aliyun_asr.region}>
                {#each aliyunRegionOptions as option}
                  <option value={option.value}>{t(option.labelKey)}</option>
                {/each}
              </select>
              {#if fieldError("aliyun_asr.region")}<small class="field-error">{fieldError("aliyun_asr.region")}</small>{/if}
            </label>
            <label class:field-invalid={Boolean(fieldError("aliyun_asr.model"))}>
              <span>{t("model")}</span>
              <input autocomplete="off" bind:value={config.aliyun_asr.model} />
              {#if fieldError("aliyun_asr.model")}<small class="field-error">{fieldError("aliyun_asr.model")}</small>{/if}
              <small class="field-hint">{t("aliyunModelHint")}</small>
            </label>
            <label class:field-invalid={Boolean(fieldError("aliyun_asr.websocket_url"))}>
              <span>{t("aliyunWebsocketUrl")}</span>
              <input autocomplete="off" bind:value={config.aliyun_asr.websocket_url} />
              {#if fieldError("aliyun_asr.websocket_url")}<small class="field-error">{fieldError("aliyun_asr.websocket_url")}</small>{/if}
              <small class="field-hint">{t("aliyunWebsocketUrlHint")}</small>
            </label>
            <label class:field-invalid={Boolean(fieldError("aliyun_asr.language_hint"))}>
              <span>{t("aliyunInputLanguage")}</span>
              <select bind:value={config.aliyun_asr.language_hint}>
                {#each aliyunLanguageOptions as option}
                  <option value={option.value}>{t(option.labelKey)}</option>
                {/each}
              </select>
              {#if fieldError("aliyun_asr.language_hint")}<small class="field-error">{fieldError("aliyun_asr.language_hint")}</small>{/if}
              <small class="field-hint">{t("aliyunInputLanguageHint")}</small>
            </label>
          {:else}
            <label class:field-invalid={Boolean(fieldError("request.language"))}>
              <span>{t("asrInputLanguage")}</span>
              <select bind:value={config.request.language}>
                {#each asrLanguageOptions as option}
                  <option value={option.value}>{t(option.labelKey)}</option>
                {/each}
              </select>
              {#if fieldError("request.language")}<small class="field-error">{fieldError("request.language")}</small>{/if}
              <small class="field-hint">{t("asrInputLanguageHint")}</small>
            </label>
          {/if}
        </div>
      </AdvancedSettings>
    </div>
  </section>
  <section class="settings-group">
    <div class="settings-group-heading">
      <h3>{t("llmApiSettings")}</h3>
      <p>{t("llmApiSettingsDescription")}</p>
    </div>
    <div id="settings-llm-api" class="form-panel">
      <div class="section-heading with-actions">
        <div class="section-heading-copy">
          <h3>{t("llmApiOptionalTitle")}</h3>
          <p>{t("llmApiOptionalDescription")}</p>
          <SettingTags tags={[t("tagOptional"), t("tagSentToService")]} />
        </div>
      </div>
      <div class="optional-config-summary">
        <span>{llmApiStatusText}</span>
        <small>{t("llmApiOptionalUses")}</small>
      </div>
      {#if llmTestStatusText}
        <div class:testing={testingLlm} class="llm-test-status" aria-live="polite">
          <ShieldCheck size={16} />
          <span>{llmTestStatusText}</span>
        </div>
      {/if}
      <label class="check">
        <input
          type="checkbox"
          checked={config.llm_post_edit.enabled}
          onchange={(event) => {
            config.llm_post_edit.enabled = event.currentTarget.checked;
          }}
        />
        <span class="check-copy">
          <span>{t("enablePolishing")}</span>
          {#if !hasLlmApiConfig}<small>{t("llmApiRequiredForPolishing")}</small>{/if}
        </span>
      </label>
      <p class="field-hint">{t("llmPolishingDataHint")}</p>
      <div id="llm-api-config-fields" class="llm-api-config-fields">
        <div class="form-grid">
          <label class:field-invalid={Boolean(fieldError("llm_post_edit.base_url"))}>
            <span>Base URL</span>
            <input bind:value={config.llm_post_edit.base_url} />
            {#if fieldError("llm_post_edit.base_url")}<small class="field-error">{fieldError("llm_post_edit.base_url")}</small>{/if}
            <small class="field-hint">{t("llmApiBaseUrlHint")}</small>
          </label>
          <label class:field-invalid={Boolean(fieldError("llm_post_edit.api_key"))}>
            <span>API Key</span>
            <input type="password" autocomplete="off" bind:value={config.llm_post_edit.api_key} />
            {#if fieldError("llm_post_edit.api_key")}<small class="field-error">{fieldError("llm_post_edit.api_key")}</small>{/if}
            <small class="field-hint">{t("llmApiKeyHint")}</small>
          </label>
          <label class:field-invalid={Boolean(fieldError("llm_post_edit.model"))}>
            <span>{t("model")}</span>
            <input bind:value={config.llm_post_edit.model} />
            {#if fieldError("llm_post_edit.model")}<small class="field-error">{fieldError("llm_post_edit.model")}</small>{/if}
            <small class="field-hint">{t("llmApiModelHint")}</small>
          </label>
        </div>
        <AdvancedSettings
          title={t("llmAdvancedSettings")}
          description={t("llmAdvancedSettingsHint")}
          expanded={llmAdvancedExpanded}
          onToggle={() => (showLlmAdvanced = !showLlmAdvanced)}
        >
          <div class="form-grid">
            <label class:field-invalid={Boolean(fieldError("llm_post_edit.thinking_strategy"))}>
              <span>{t("llmThinkingStrategy")}</span>
              <select bind:value={config.llm_post_edit.thinking_strategy}>
                {#each llmThinkingStrategyOptions as option}
                  <option value={option.value}>{t(option.labelKey)}</option>
                {/each}
              </select>
              {#if fieldError("llm_post_edit.thinking_strategy")}<small class="field-error">{fieldError("llm_post_edit.thinking_strategy")}</small>{/if}
              <small class="field-hint">{t("llmThinkingStrategyHint")}</small>
            </label>
          </div>
        </AdvancedSettings>
        <button class="test-button" type="button" onclick={onTestLlmConfig} disabled={testingLlm}>
          <ShieldCheck size={16} />{testingLlm ? t("testingConnection") : t("testConnection")}
        </button>
      </div>
    </div>
  </section>
</section>

<style>
  .optional-config-summary {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    min-width: 0;
    padding: 10px 12px;
    color: var(--text-secondary);
    background: #f8fbff;
    border: 1px solid var(--border);
    border-radius: 10px;
    font-size: 13px;
    line-height: 1.45;
  }

  .optional-config-summary span {
    min-width: 0;
    color: var(--text-main);
    font-weight: 800;
    overflow-wrap: anywhere;
  }

  .optional-config-summary small {
    color: var(--text-secondary);
    overflow-wrap: anywhere;
  }

  .llm-test-status {
    display: grid;
    grid-template-columns: 18px minmax(0, 1fr);
    align-items: start;
    gap: 8px;
    margin: 8px 0 0;
    padding: 9px 11px;
    color: var(--text-main);
    background: #f8fbff;
    border: 1px solid var(--border);
    border-radius: 8px;
    font-size: 13px;
    line-height: 1.45;
  }

  .llm-test-status.testing {
    color: #174ea6;
    background: #f1f6ff;
    border-color: #bfd6ff;
  }

  .llm-test-status span {
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .llm-api-config-fields {
    display: grid;
    gap: 14px;
  }

  .setup-note {
    margin: 8px 0 0;
    color: #8a4b00;
    font-size: 13px;
    line-height: 1.45;
  }

  .auth-gate-card {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: start;
    gap: 16px;
    padding: 16px;
    background: linear-gradient(180deg, #fffaf3 0%, #fffefd 100%);
    border: 1px solid #f7cf96;
    border-radius: 14px;
  }

  .auth-gate-copy {
    display: grid;
    gap: 10px;
    min-width: 0;
  }

  .auth-gate-card strong {
    color: var(--text-main);
  }

  .auth-gate-card p {
    margin: 4px 0 0;
    color: var(--text-secondary);
    font-size: 14px;
  }

  .starter-steps {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .starter-steps li {
    display: grid;
    grid-template-columns: 26px minmax(0, 1fr);
    align-items: start;
    gap: 8px;
    min-width: 0;
    padding: 10px;
    background: rgba(255, 255, 255, 0.72);
    border: 1px solid rgba(247, 207, 150, 0.72);
    border-radius: 12px;
  }

  .starter-steps li > span {
    display: grid;
    width: 26px;
    height: 26px;
    place-items: center;
    color: #8a4b00;
    background: #ffedd5;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 900;
  }

  .starter-steps li div {
    display: grid;
    gap: 3px;
    min-width: 0;
  }

  .starter-steps li strong,
  .starter-steps li small {
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .starter-steps li strong {
    font-size: 13px;
    line-height: 1.25;
  }

  .starter-steps li small {
    color: #715536;
    font-size: 12px;
    line-height: 1.35;
  }

  .setup-actions {
    display: flex;
    flex: 0 0 auto;
    flex-wrap: wrap;
    gap: 10px;
  }

  .setup-actions button {
    min-height: 36px;
    padding: 0 12px;
    color: #ffffff;
    background: var(--primary);
    border-radius: 10px;
    font-weight: 600;
  }

  .setup-actions .secondary {
    color: var(--primary);
    background: var(--primary-light);
  }

  @media (max-width: 920px) {
    .auth-gate-card {
      grid-template-columns: 1fr;
    }

    .setup-actions {
      justify-content: stretch;
    }

    .setup-actions button {
      flex: 1 1 150px;
    }
  }

  @media (max-width: 720px) {
    .starter-steps {
      grid-template-columns: 1fr;
    }
  }
</style>
