<script lang="ts">
  import SettingTags from "$lib/components/common/SettingTags.svelte";
  import SetupStatusCard, {
    type SetupStatusItem,
    type SetupStatusWarning,
  } from "$lib/components/overview/SetupStatusCard.svelte";
  import type { CopyKey } from "$lib/i18n";
  import type { AppConfig } from "$lib/types/app";
  import { ChevronDown, ChevronUp, ShieldCheck } from "lucide-svelte";

  type Translate = (key: CopyKey, values?: Record<string, string>) => string;

  const asrLanguageOptions: Array<{ value: string; labelKey: CopyKey }> = [
    { value: "", labelKey: "asrLanguageAuto" },
    { value: "zh-CN", labelKey: "asrLanguageZhCN" },
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

  type Props = {
    config: AppConfig;
    llmApiConfigVisible: boolean;
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
    fieldError: (field: string) => string;
    setupRequiredMessage: () => string;
    setupActionText: (action: string) => string;
    formatHotkey: (value: string) => string;
    onScrollToSettingsPanel: (id: string) => void;
    onOpenSetupGuide: () => void;
    onRefreshSetupStatus: () => void;
    onSetupAction: (action: string) => void;
    onTestAsrConfig: () => void;
    onTestLlmConfig: () => void;
  };

  let {
    config = $bindable<AppConfig>(),
    llmApiConfigVisible = $bindable<boolean>(),
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
    fieldError,
    setupRequiredMessage,
    setupActionText,
    formatHotkey,
    onScrollToSettingsPanel,
    onOpenSetupGuide,
    onRefreshSetupStatus,
    onSetupAction,
    onTestAsrConfig,
    onTestLlmConfig,
  }: Props = $props();
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
      <h3>{t("apiConfigPageTitle")}</h3>
      <p>{t("apiConfigPageDescription")}</p>
    </div>
    <div id="settings-auth" class="form-panel">
      <div class="section-heading with-actions">
        <div class="section-heading-copy">
          <h3>{t("doubaoAuth")}</h3>
          <p>{t("doubaoAuthRequiredHint")}</p>
          <SettingTags tags={[{ label: t("tagRequired"), tone: "required" }, t("tagSentToService")]} />
          {#if requiresAsrAuth}
            <p class="setup-note">{setupRequiredMessage()}</p>
            <button class="link-button" type="button" onclick={onOpenSetupGuide}>{t("setupGuideCta")}</button>
          {/if}
        </div>
        <div class="settings-inline-actions">
          <button class="test-button" type="button" onclick={onTestAsrConfig} disabled={testingAsr}>
            <ShieldCheck size={16} />{testingAsr ? t("testingConnection") : t("testConnection")}
          </button>
        </div>
      </div>
      <div class="form-grid">
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
      </div>
    </div>
    <div id="settings-asr-language" class="form-panel">
      <div class="section-heading">
        <div class="section-heading-copy">
          <h3>{t("asrLanguageTitle")}</h3>
          <p>{t("asrLanguageDescription")}</p>
          <SettingTags tags={[t("tagOptional"), t("tagSentToService")]} />
        </div>
      </div>
      <div class="form-grid">
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
      </div>
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
        <button
          class="test-button config-toggle-button"
          type="button"
          aria-controls="llm-api-config-fields"
          aria-expanded={llmApiConfigVisible}
          onclick={() => (llmApiConfigVisible = !llmApiConfigVisible)}
        >
          {#if llmApiConfigVisible}
            <ChevronUp size={16} />{t("hideLlmConfig")}
          {:else}
            <ChevronDown size={16} />{t("expandLlmConfig")}
          {/if}
        </button>
      </div>
      <div class="optional-config-summary">
        <span>{llmApiStatusText}</span>
        <small>{t("llmApiOptionalUses")}</small>
      </div>
      <label class="check">
        <input
          type="checkbox"
          checked={config.llm_post_edit.enabled}
          onchange={(event) => {
            config.llm_post_edit.enabled = event.currentTarget.checked;
            if (event.currentTarget.checked) llmApiConfigVisible = true;
          }}
        />
        <span class="check-copy">
          <span>{t("enablePolishing")}</span>
          {#if !hasLlmApiConfig}<small>{t("llmApiRequiredForPolishing")}</small>{/if}
        </span>
      </label>
      <div id="llm-api-config-fields" class="llm-api-config-fields" hidden={!llmApiConfigVisible}>
        <div class="form-grid">
          <label class:field-invalid={Boolean(fieldError("llm_post_edit.base_url"))}>
            <span>Base URL</span>
            <input bind:value={config.llm_post_edit.base_url} />
            {#if fieldError("llm_post_edit.base_url")}<small class="field-error">{fieldError("llm_post_edit.base_url")}</small>{/if}
          </label>
          <label class:field-invalid={Boolean(fieldError("llm_post_edit.api_key"))}>
            <span>API Key</span>
            <input type="password" autocomplete="off" bind:value={config.llm_post_edit.api_key} />
            {#if fieldError("llm_post_edit.api_key")}<small class="field-error">{fieldError("llm_post_edit.api_key")}</small>{/if}
          </label>
          <label class:field-invalid={Boolean(fieldError("llm_post_edit.model"))}>
            <span>{t("model")}</span>
            <input bind:value={config.llm_post_edit.model} />
            {#if fieldError("llm_post_edit.model")}<small class="field-error">{fieldError("llm_post_edit.model")}</small>{/if}
          </label>
        </div>
        <button class="test-button" type="button" onclick={onTestLlmConfig} disabled={testingLlm}>
          <ShieldCheck size={16} />{testingLlm ? t("testingConnection") : t("testConnection")}
        </button>
      </div>
    </div>
  </section>
</section>

<style>
  .config-toggle-button {
    min-width: 112px;
  }

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

  .llm-api-config-fields {
    display: grid;
    gap: 14px;
  }

  .llm-api-config-fields[hidden] {
    display: none;
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
