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
        <label class="check">
          <input type="checkbox" bind:checked={config.llm_post_edit.use_recent_context} />
          <span class="check-copy">
            <span>{t("useRecentContextForLlm")}</span>
            <small>{t("tagSentToService")}</small>
          </span>
        </label>
      </div>
      <p class="field-hint">{t("recentContextHint")}</p>
      <p class="field-hint">{t("recentContextForLlmHint")}</p>
      {#if config.llm_post_edit.use_recent_context && !config.context.enable_recent_context}
        <div class="inline-warning">
          <AlertCircle size={16} />
          <span>{t("recentContextForLlmNeedsRecentContext")}</span>
        </div>
      {/if}
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
          <button class="test-button" type="button" onclick={onRestoreDefaultPrompt}><Sparkles size={16} />{t("restoreDefaultPrompt")}</button>
          <button class="test-button" type="button" onclick={onPreviewFinalPrompt}><FileText size={16} />{t("previewFinalPrompt")}</button>
        </div>
      </div>
      <label class:field-invalid={Boolean(fieldError("llm_post_edit.user_prompt_template"))}>
        <span>{t("userPromptTemplate")}</span>
        <textarea bind:value={config.llm_post_edit.user_prompt_template}></textarea>
        {#if fieldError("llm_post_edit.user_prompt_template")}<small class="field-error">{fieldError("llm_post_edit.user_prompt_template")}</small>{/if}
      </label>
      <div class="form-grid">
        <label class:field-invalid={Boolean(fieldError("llm_post_edit.min_chars"))}>
          <span>{t("minChars")}</span>
          <input type="number" min="0" max="10000" step="1" bind:value={config.llm_post_edit.min_chars} />
          {#if fieldError("llm_post_edit.min_chars")}<small class="field-error">{fieldError("llm_post_edit.min_chars")}</small>{/if}
          <small class="field-hint">{t("minCharsHint")}</small>
        </label>
      </div>
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
</style>
