<script lang="ts">
  import SettingTags from "$lib/components/common/SettingTags.svelte";
  import type { AppConfig, SelectableHotwordCandidate } from "$lib/types/app";
  import type { CopyKey } from "$lib/i18n";
  import { AlertCircle, Check, FileText, Info, Sparkles, Trash2 } from "lucide-svelte";

  type Translate = (key: CopyKey, values?: Record<string, string>) => string;

  type Props = {
    config: AppConfig;
    autoHotwordCandidates: SelectableHotwordCandidate[];
    t: Translate;
    generatingAutoHotwords: boolean;
    clearingAutoHotwordHistory: boolean;
    autoHotwordError: string;
    showAutoHotwordDetails: boolean;
    hasLlmApiConfig: boolean;
    hotwordCount: number;
    acceptedAutoHotwordCount: number;
    selectedAutoHotwordCount: number;
    autoHotwordStatusText: string;
    fieldError: (field: string) => string;
    candidateConfidenceLabel: (confidence: number) => string;
    onUpdateHotwords: (value: string) => void;
    onTidyHotwords: () => void;
    onClearHotwords: () => void;
    onUpdatePromptContext: (value: string) => void;
    onOptionEnabledNotice: (key: "enable_recent_context", enabled: boolean) => void;
    onRestoreDefaultPrompt: () => void;
    onPreviewFinalPrompt: () => void;
    onOpenLlmApiSettings: () => void;
    onGenerateAutoHotwords: () => void;
    onClearAutoHotwordHistory: () => void;
    onRefreshAutoHotwordStatus: () => void;
    onUpdateAcceptedAutoHotwords: (value: string) => void;
    onTidyAcceptedAutoHotwords: () => void;
    onClearAcceptedAutoHotwords: () => void;
    onApplySelectedAutoHotwords: () => void;
  };

  let {
    config = $bindable<AppConfig>(),
    autoHotwordCandidates = $bindable<SelectableHotwordCandidate[]>(),
    t,
    generatingAutoHotwords,
    clearingAutoHotwordHistory,
    autoHotwordError,
    showAutoHotwordDetails,
    hasLlmApiConfig,
    hotwordCount,
    acceptedAutoHotwordCount,
    selectedAutoHotwordCount,
    autoHotwordStatusText,
    fieldError,
    candidateConfidenceLabel,
    onUpdateHotwords,
    onTidyHotwords,
    onClearHotwords,
    onUpdatePromptContext,
    onOptionEnabledNotice,
    onRestoreDefaultPrompt,
    onPreviewFinalPrompt,
    onOpenLlmApiSettings,
    onGenerateAutoHotwords,
    onClearAutoHotwordHistory,
    onRefreshAutoHotwordStatus,
    onUpdateAcceptedAutoHotwords,
    onTidyAcceptedAutoHotwords,
    onClearAcceptedAutoHotwords,
    onApplySelectedAutoHotwords,
  }: Props = $props();

  let promptEditorOpen = $state(false);
</script>

<section class="settings-stack">
  <section class="settings-group">
    <div class="settings-group-heading">
      <h3>{t("hotwordsPageTitle")}</h3>
      <p>{t("hotwordsPageDescription")}</p>
    </div>
    <div id="settings-context" class="form-panel">
      <div class="section-heading with-actions">
        <div class="section-heading-copy">
          <h3>{t("hotwordsCommonTitle")}</h3>
          <p>{t("hotwordsCommonDescription")}</p>
          <SettingTags tags={[t("tagSentToService")]} />
        </div>
        <div class="settings-inline-actions">
          <button class="test-button" type="button" onclick={onTidyHotwords}><Sparkles size={16} />{t("tidyHotwords")}</button>
          <button class="test-button" type="button" onclick={onClearHotwords}><Trash2 size={16} />{t("clearHotwords")}</button>
        </div>
      </div>
      <p class="field-hint">{t("customHotwordCount", { count: String(hotwordCount) })}</p>
      <label><span>{t("customHotwords")}</span><textarea value={config.context.hotwords.join("\n")} oninput={(event) => onUpdateHotwords(event.currentTarget.value)}></textarea></label>
      <p class="field-hint">{t("hotwordsPrivacyHint")}</p>
    </div>
    <div id="settings-llm-prompt" class="form-panel">
      <div class="section-heading with-actions">
        <div class="section-heading-copy">
          <h3>{t("polishingPromptTitle")}</h3>
          <p>{t("polishingPromptDescription")}</p>
          <SettingTags tags={[t("tagOptional")]} />
          {#if !config.llm_post_edit.enabled}
            <p class="field-hint">{t("polishingPromptInactiveHint")}</p>
          {/if}
        </div>
        <div class="settings-inline-actions">
          <button class="test-button" type="button" onclick={() => (promptEditorOpen = !promptEditorOpen)}><FileText size={16} />{promptEditorOpen ? t("hidePromptEditor") : t("editPrompt")}</button>
          <button class="test-button" type="button" onclick={onRestoreDefaultPrompt}><Sparkles size={16} />{t("restoreDefaultPrompt")}</button>
          <button class="test-button" type="button" onclick={onPreviewFinalPrompt}><FileText size={16} />{t("previewFinalPrompt")}</button>
        </div>
      </div>
      {#if promptEditorOpen}
        <label class:field-invalid={Boolean(fieldError("llm_post_edit.user_prompt_template"))}>
          <span>{t("userPromptTemplate")}</span>
          <textarea bind:value={config.llm_post_edit.user_prompt_template}></textarea>
          {#if fieldError("llm_post_edit.user_prompt_template")}<small class="field-error">{fieldError("llm_post_edit.user_prompt_template")}</small>{/if}
        </label>
      {:else}
        <div class="prompt-summary">
          <FileText size={16} />
          <span>{t("promptEditorCollapsedHint")}</span>
        </div>
      {/if}
      <div class="form-grid">
        <label><span>{t("minChars")}</span><input type="number" bind:value={config.llm_post_edit.min_chars} /></label>
      </div>
    </div>
    <div id="settings-prompt-context" class="form-panel">
      <div class="section-heading">
        <div class="section-heading-copy">
          <h3>{t("sceneContext")}</h3>
          <p>{t("sceneContextDescription")}</p>
          <SettingTags tags={[t("tagLocalOnly"), t("tagPrivacySensitive")]} />
        </div>
      </div>
      <label><span>{t("promptContext")}</span><textarea value={config.context.prompt_context.map((item) => item.text).join("\n")} oninput={(event) => onUpdatePromptContext(event.currentTarget.value)}></textarea></label>
      <div class="toggle-grid">
        <label class="check"><input type="checkbox" bind:checked={config.context.enable_recent_context} onchange={(event) => onOptionEnabledNotice("enable_recent_context", event.currentTarget.checked)} />{t("useRecentContext")}</label>
      </div>
      <p class="field-hint">{t("recentContextHint")}</p>
    </div>
    <div id="settings-auto-hotwords" class="form-panel auto-hotwords-panel">
        <div class="section-heading with-actions">
          <div class="section-heading-copy">
            <h3>{t("autoHotwordsTitle")}</h3>
            <p>{t("autoHotwordsDescription")}</p>
            <SettingTags tags={[t("tagOptional"), t("tagLocalOnly"), t("tagSentToService")]} />
          </div>
          <div class="settings-inline-actions">
            {#if showAutoHotwordDetails}
              <button class="test-button" type="button" onclick={onGenerateAutoHotwords} disabled={generatingAutoHotwords || !hasLlmApiConfig || !config.auto_hotwords.enabled}>
                <Sparkles size={16} />{generatingAutoHotwords ? t("autoHotwordsGenerating") : t("autoHotwordsGenerate")}
              </button>
              <button class="test-button" type="button" onclick={onClearAutoHotwordHistory} disabled={clearingAutoHotwordHistory}>
                <Trash2 size={16} />{clearingAutoHotwordHistory ? t("autoHotwordsClearing") : t("autoHotwordsClearHistory")}
              </button>
            {/if}
          </div>
        </div>
        <div class="toggle-grid">
          <label class="check"><input type="checkbox" bind:checked={config.auto_hotwords.enabled} />{t("autoHotwordsEnabled")}</label>
        </div>
        <p class="field-hint">{t("autoHotwordsPrivacyHint")}</p>
        {#if showAutoHotwordDetails && !hasLlmApiConfig}
          <div class="inline-warning">
            <AlertCircle size={16} />
            <span>{t("autoHotwordsNeedsLlmApi")}</span>
            <button class="link-button" type="button" onclick={onOpenLlmApiSettings}>{t("goApiConfig")}</button>
          </div>
        {/if}
        {#if showAutoHotwordDetails}
          <div class="auto-hotword-list-editor">
            <div class="auto-hotword-list-head">
              <div>
                <strong>{t("autoHotwordsAcceptedTitle", { count: String(acceptedAutoHotwordCount) })}</strong>
                <span>{t("autoHotwordsAcceptedDescription")}</span>
              </div>
              <div class="settings-inline-actions">
                <button class="test-button" type="button" onclick={onTidyAcceptedAutoHotwords}><Sparkles size={16} />{t("tidyHotwords")}</button>
                <button class="test-button" type="button" onclick={onClearAcceptedAutoHotwords}><Trash2 size={16} />{t("autoHotwordsAcceptedClear")}</button>
              </div>
            </div>
            <label>
              <span>{t("autoHotwordsAcceptedList")}</span>
              <textarea value={config.auto_hotwords.accepted_hotwords.join("\n")} oninput={(event) => onUpdateAcceptedAutoHotwords(event.currentTarget.value)}></textarea>
            </label>
            <p class="field-hint">{t("autoHotwordsAcceptedHint")}</p>
          </div>
          <div class="auto-hotword-status">
            <Info size={16} />
            <span>{autoHotwordStatusText}</span>
            <button class="link-button" type="button" onclick={onRefreshAutoHotwordStatus}>{t("refreshStatus")}</button>
          </div>
          {#if autoHotwordError}
            <p class="field-error">{autoHotwordError}</p>
          {/if}
          {#if autoHotwordCandidates.length > 0}
            <div class="auto-hotword-candidates">
              <div class="candidate-list-head">
                <strong>{t("autoHotwordsCandidatesTitle", { count: String(autoHotwordCandidates.length) })}</strong>
                <button class="test-button" type="button" onclick={onApplySelectedAutoHotwords}>
                  <Check size={16} />{t("autoHotwordsApplySelected", { count: String(selectedAutoHotwordCount) })}
                </button>
              </div>
              {#each autoHotwordCandidates as candidate}
                <label class="candidate-row">
                  <input type="checkbox" bind:checked={candidate.selected} />
                  <span class="candidate-copy">
                    <strong>{candidate.word}</strong>
                    <small>{candidate.category || t("autoHotwordsUnknownCategory")} · {candidateConfidenceLabel(candidate.confidence)} · {t("autoHotwordsSourceCount", { count: String(candidate.source_count) })}</small>
                    <span>{candidate.reason}</span>
                  </span>
                </label>
              {/each}
            </div>
          {/if}
        {/if}
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

  .form-grid,
  .toggle-grid {
    display: grid;
    align-items: start;
    gap: 16px 14px;
  }

  .form-grid {
    grid-template-columns: repeat(auto-fit, minmax(min(320px, 100%), 1fr));
  }

  .toggle-grid {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px 18px;
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

  input,
  textarea {
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

  textarea {
    min-height: 84px;
    padding: 10px 12px;
    resize: vertical;
  }

  input:focus,
  textarea:focus,
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

  .field-invalid textarea {
    border-color: var(--danger);
    background: #fff7f7;
  }

  .field-error {
    color: var(--danger);
    font-size: 12px;
    line-height: 1.35;
  }

  .inline-warning {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    min-width: 0;
    padding: 10px 12px;
    color: #8a4b00;
    background: #fffaf3;
    border: 1px solid #f4d7ad;
    border-radius: 10px;
    font-size: 13px;
    line-height: 1.45;
  }

  .inline-warning :global(svg) {
    flex: 0 0 auto;
  }

  .prompt-summary {
    display: grid;
    gap: 12px;
    padding: 14px;
    background: #fbfdff;
    border: 1px solid var(--border);
    border-radius: 12px;
  }

  .prompt-summary {
    grid-template-columns: auto minmax(0, 1fr);
    align-items: center;
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.45;
  }

  .prompt-summary :global(svg) {
    color: var(--primary);
  }

  .link-button {
    min-height: 36px;
    padding: 0 12px;
    color: var(--primary);
    background: var(--primary-light);
    border-radius: 10px;
    font-weight: 600;
  }

  .auto-hotword-list-editor {
    display: grid;
    gap: 12px;
    padding: 14px;
    background: #fbfdff;
    border: 1px solid var(--border);
    border-radius: 10px;
  }

  .auto-hotword-list-head,
  .candidate-list-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 10px;
  }

  .auto-hotword-list-head > div:first-child {
    display: grid;
    gap: 4px;
    min-width: 0;
  }

  .auto-hotword-list-head strong,
  .candidate-list-head strong {
    color: var(--text-main);
    font-size: 14px;
    font-weight: 800;
  }

  .auto-hotword-list-head span {
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.45;
    overflow-wrap: anywhere;
  }

  .auto-hotword-status {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    padding: 10px 12px;
    color: var(--text-secondary);
    background: #f8fbff;
    border: 1px solid var(--border);
    border-radius: 8px;
    font-size: 13px;
    line-height: 1.4;
  }

  .auto-hotword-status :global(svg) {
    flex: 0 0 auto;
    color: var(--primary);
  }

  .auto-hotword-status span,
  .candidate-copy strong,
  .candidate-copy small,
  .candidate-copy span {
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .auto-hotword-candidates {
    display: grid;
    gap: 10px;
  }

  .candidate-row {
    display: grid !important;
    grid-template-columns: auto minmax(0, 1fr);
    align-items: start;
    gap: 10px;
    padding: 12px;
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
  }

  .candidate-row:hover {
    border-color: rgba(47, 128, 237, 0.4);
    background: #f8fbff;
  }

  .candidate-row input {
    width: 16px;
    height: 16px;
    margin-top: 2px;
  }

  .candidate-copy {
    display: grid;
    gap: 4px;
    min-width: 0;
  }

  .candidate-copy strong {
    color: var(--text-main);
    font-size: 14px;
  }

  .candidate-copy small {
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.35;
  }

  .candidate-copy span {
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.4;
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
    .form-grid {
      grid-template-columns: 1fr;
    }

    .toggle-grid {
      display: grid;
      grid-template-columns: 1fr;
    }

    .check {
      width: 100%;
    }
  }
</style>
