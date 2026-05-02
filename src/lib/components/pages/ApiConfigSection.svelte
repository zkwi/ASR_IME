<script lang="ts">
  import AdvancedPanel from "$lib/components/common/AdvancedPanel.svelte";
  import SettingTags from "$lib/components/common/SettingTags.svelte";
  import SetupStatusCard, {
    type SetupStatusItem,
    type SetupStatusWarning,
  } from "$lib/components/overview/SetupStatusCard.svelte";
  import SettingsToolbar from "$lib/components/settings/SettingsToolbar.svelte";
  import type { AppConfig } from "$lib/types/app";
  import type { CopyKey } from "$lib/i18n";
  import { ChevronDown, ChevronUp, ShieldCheck } from "lucide-svelte";

  type Translate = (key: CopyKey, values?: Record<string, string>) => string;

  type Props = {
    config: AppConfig;
    llmApiConfigVisible: boolean;
    t: Translate;
    saving: boolean;
    settingsDirty: boolean;
    toolbarMessage: string;
    advancedOpen: boolean;
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
    onReload: () => void;
    onToggleAdvanced: () => void;
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
    saving,
    settingsDirty,
    toolbarMessage,
    advancedOpen,
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
    onReload,
    onToggleAdvanced,
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
      <div>
        <strong>{t("authGateTitle")}</strong>
        <p>{!configExists ? t("setupMissingFile") : t("authGateDescription")}</p>
      </div>
      <div class="setup-actions">
        <button type="button" onclick={() => onScrollToSettingsPanel("settings-auth")}>{t("setupCta")}</button>
        <button type="button" class="secondary" onclick={onOpenSetupGuide}>{t("setupGuideCta")}</button>
      </div>
    </section>
  {/if}
  <SettingsToolbar
    title={t("settingsActionTitle")}
    hint={t("settingsActionHint")}
    statusMessage={toolbarMessage}
    reloadLabel={t("reload")}
    {saving}
    dirty={settingsDirty}
    onReload={onReload}
  />
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
  <AdvancedPanel
    title={advancedOpen ? t("advancedSettings") : t("basicSettings")}
    description={advancedOpen ? t("advancedSettingsHint") : t("apiConfigBasicHint")}
    open={advancedOpen}
    showLabel={t("showAdvancedSettings")}
    hideLabel={t("hideAdvancedSettings")}
    onToggle={onToggleAdvanced}
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
        <label><span>{t("resourceId")}</span><input bind:value={config.auth.resource_id} /></label>
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
    {#if advancedOpen}
      <div id="settings-request" class="form-panel">
        <div class="section-heading">
          <div class="section-heading-copy">
            <h3>{t("asrConnectionParams")}</h3>
            <p>{t("asrConnectionParamsDescription")}</p>
            <SettingTags tags={[t("tagAdvanced")]} />
          </div>
        </div>
        <div class="form-grid">
          <label class:field-invalid={Boolean(fieldError("request.final_result_timeout_seconds"))}>
            <span>{t("finalTimeout")}</span>
            <input type="number" bind:value={config.request.final_result_timeout_seconds} />
            {#if fieldError("request.final_result_timeout_seconds")}<small class="field-error">{fieldError("request.final_result_timeout_seconds")}</small>{/if}
            <small class="field-hint">{t("finalTimeoutHint")}</small>
          </label>
          <label><span>{t("model")}</span><input bind:value={config.request.model_name} /></label>
        </div>
        <label class:field-invalid={Boolean(fieldError("request.ws_url"))}>
          <span>{t("websocketUrl")}</span>
          <input bind:value={config.request.ws_url} />
          {#if fieldError("request.ws_url")}<small class="field-error">{fieldError("request.ws_url")}</small>{/if}
        </label>
      </div>
    {/if}
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
        {#if advancedOpen}
          <div class="advanced-subpanel">
            <div class="section-heading">
              <div class="section-heading-copy">
                <h3>{t("llmAdvancedParams")}</h3>
                <p>{t("llmAdvancedParamsDescription")}</p>
              </div>
            </div>
            <div class="form-grid">
              <label class:field-invalid={Boolean(fieldError("llm_post_edit.timeout_seconds"))}>
                <span>{t("timeout")}</span>
                <input type="number" bind:value={config.llm_post_edit.timeout_seconds} />
                {#if fieldError("llm_post_edit.timeout_seconds")}<small class="field-error">{fieldError("llm_post_edit.timeout_seconds")}</small>{/if}
              </label>
            </div>
            <label class="check">
              <input type="checkbox" bind:checked={config.llm_post_edit.enable_thinking} />
              <span class="check-copy">
                <span>{t("enableThinking")}</span>
                <small>{t("enableThinkingHint")}</small>
              </span>
            </label>
          </div>
        {/if}
      </div>
    </div>
  </section>
</section>

<style>
  .settings-stack {
    display: grid;
    gap: 18px;
    max-width: 1040px;
  }

  .settings-group {
    display: grid;
    gap: 12px;
  }

  .settings-group-heading {
    display: grid;
    gap: 4px;
    padding: 0 2px;
  }

  .settings-group-heading h3,
  .section-heading h3 {
    margin: 0;
    color: var(--text-main);
    font-weight: 800;
  }

  .settings-group-heading h3 {
    font-size: 20px;
  }

  .settings-group-heading p,
  .section-heading p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .form-panel {
    display: grid;
    gap: 14px;
    min-width: 0;
    padding: 18px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 18px;
  }

  .form-panel[id^="settings-"] {
    scroll-margin-top: 86px;
  }

  .form-panel label {
    display: grid;
    align-content: start;
    gap: 8px;
    color: var(--text-secondary);
    font-size: 14px;
  }

  .section-heading {
    display: grid;
    gap: 4px;
  }

  .section-heading h3 {
    margin-bottom: 6px;
    font-size: 16px;
  }

  .section-heading.with-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .section-heading.with-actions > div,
  .section-heading-copy {
    min-width: 0;
  }

  .section-heading-copy {
    flex: 1 1 320px;
  }

  .form-grid {
    display: grid;
    align-items: start;
    gap: 16px 14px;
  }

  .form-grid {
    grid-template-columns: repeat(auto-fit, minmax(min(320px, 100%), 1fr));
  }

  .check {
    display: inline-flex !important;
    align-items: center;
    flex: 0 1 auto;
    gap: 10px;
    min-height: 38px;
    width: fit-content;
    max-width: 100%;
    min-width: 0;
    color: var(--text-main);
    font-weight: 700;
    line-height: 1.35;
    white-space: normal;
    overflow-wrap: anywhere;
  }

  .check input {
    flex: 0 0 auto;
    width: 18px;
    min-height: 18px;
    accent-color: var(--primary);
  }

  .check-copy {
    display: grid;
    gap: 2px;
    min-width: 0;
  }

  .check-copy span {
    color: var(--text-main);
    font-size: 14px;
    font-weight: 700;
    white-space: normal;
  }

  .check-copy small {
    color: var(--text-muted);
    font-size: 11px;
    font-weight: 700;
    line-height: 1.25;
    white-space: normal;
    overflow-wrap: anywhere;
  }

  .settings-inline-actions {
    display: flex;
    flex: 0 0 auto;
    flex-wrap: wrap;
    gap: 10px;
    justify-content: flex-end;
    min-width: 0;
  }

  .test-button {
    display: inline-flex;
    flex: 0 0 auto;
    align-items: center;
    justify-content: center;
    gap: 7px;
    min-width: 96px;
    min-height: 36px;
    padding: 0 14px;
    color: var(--primary);
    background: var(--primary-light);
    border: 1px solid rgba(47, 128, 237, 0.18);
    border-radius: 12px;
    font-size: 13px;
    font-weight: 800;
    line-height: 1.2;
    white-space: normal;
    overflow-wrap: anywhere;
  }

  .test-button:disabled {
    cursor: wait;
    opacity: 0.68;
  }

  .config-toggle-button {
    min-width: 112px;
  }

  input {
    width: 100%;
    min-height: 38px;
    padding: 0 12px;
    color: var(--text-main);
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 10px;
  }

  input {
    min-width: 0;
    text-overflow: ellipsis;
  }

  input:focus,
  button:focus-visible {
    border-color: var(--primary);
    box-shadow: 0 0 0 3px rgba(47, 128, 237, 0.14);
  }

  .field-hint {
    margin: 8px 0 0;
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.45;
  }

  .field-invalid input {
    border-color: var(--danger);
    background: #fff7f7;
  }

  .field-error {
    color: var(--danger);
    font-size: 12px;
    line-height: 1.35;
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

  .advanced-subpanel {
    display: grid;
    gap: 12px;
    padding: 14px;
    background: #fbfdff;
    border: 1px solid var(--border);
    border-radius: 12px;
  }

  .setup-note {
    margin: 8px 0 0;
    color: #8a4b00;
    font-size: 13px;
    line-height: 1.45;
  }

  .auth-gate-card {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 14px 16px;
    background: #fff7ed;
    border: 1px solid #fed7aa;
    border-radius: 14px;
  }

  .auth-gate-card strong {
    color: var(--text-main);
  }

  .auth-gate-card p {
    margin: 4px 0 0;
    color: var(--text-secondary);
    font-size: 14px;
  }

  .setup-actions {
    display: flex;
    flex: 0 0 auto;
    flex-wrap: wrap;
    gap: 10px;
  }

  .setup-actions button,
  .link-button {
    min-height: 36px;
    padding: 0 12px;
    color: #ffffff;
    background: var(--primary);
    border-radius: 10px;
    font-weight: 600;
  }

  .setup-actions .secondary,
  .link-button {
    color: var(--primary);
    background: var(--primary-light);
  }

  @media (max-width: 920px) {
    .section-heading.with-actions {
      display: grid;
      grid-template-columns: 1fr;
      align-items: stretch;
    }

    .test-button {
      width: 100%;
    }

    .form-grid {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 720px) {
    .check {
      width: 100%;
    }
  }
</style>
