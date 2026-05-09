<script lang="ts">
  import { onMount } from "svelte";
  import type { CopyKey } from "$lib/i18n";
  import type { AppConfig, LocalDataStatus } from "$lib/types/app";

  type Translate = (key: CopyKey, values?: Record<string, string>) => string;

  type Props = {
    config: AppConfig;
    t: Translate;
    status: LocalDataStatus | null;
    clearingRecentContext: boolean;
    clearingAutoHotwordHistory: boolean;
    clearingUsageStats: boolean;
    onRefreshStatus: () => void;
    onClearRecentContext: () => void;
    onClearAutoHotwordHistory: () => void;
    onClearUsageStats: () => void;
  };

  let {
    config = $bindable<AppConfig>(),
    t,
    status,
    clearingRecentContext,
    clearingAutoHotwordHistory,
    clearingUsageStats,
    onRefreshStatus,
    onClearRecentContext,
    onClearAutoHotwordHistory,
    onClearUsageStats,
  }: Props = $props();

  onMount(() => {
    onRefreshStatus();
  });

  function entriesCount(count: number | undefined) {
    if (count === undefined) return t("privacyStatusLoading");
    return t("privacyEntriesCount", { count: String(count) });
  }

  function statsCount(count: number | undefined) {
    if (count === undefined) return t("privacyStatusLoading");
    return t("privacyStatsCount", { count: String(count) });
  }
</script>

<section class="privacy-stack">
  <div class="privacy-heading">
    <div>
      <h3>{t("privacyPageTitle")}</h3>
      <p>{t("privacyPageDescription")}</p>
    </div>
    <button type="button" class="secondary-action" onclick={onRefreshStatus}>{t("privacyRefreshStatus")}</button>
  </div>

  <section class="form-panel">
    <div class="section-heading">
      <h3>{t("privacyLocalDataTitle")}</h3>
      <p>{t("privacyLocalDataDescription")}</p>
    </div>

    <div class="status-strip">
      <div>
        <span>{t("privacyRecentContextData")}</span>
        <strong>{entriesCount(status?.recent_context_count)}</strong>
      </div>
      <div>
        <span>{t("privacyAutoHotwordHistoryData")}</span>
        <strong>{entriesCount(status?.auto_hotword_entry_count)}</strong>
      </div>
      <div>
        <span>{t("privacyStatsData")}</span>
        <strong>{statsCount(status?.stats_event_count)}</strong>
      </div>
    </div>

    <div class="table-scroll">
      <table>
        <thead>
          <tr>
            <th>{t("privacyDataType")}</th>
            <th>{t("privacySavedColumn")}</th>
            <th>{t("privacyLocationColumn")}</th>
            <th>{t("privacyUploadColumn")}</th>
            <th>{t("privacyActionColumn")}</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td><strong>{t("privacyRecentContextData")}</strong></td>
            <td>
              {config.context.enable_recent_context ? t("privacySavedWhenEnabled") : t("privacyDisabledNoSave")}
              <small>{entriesCount(status?.recent_context_count)}</small>
            </td>
            <td><code>context/recent_context.jsonl</code></td>
            <td>{t("privacyUploadRecentContext")}</td>
            <td>
              <div class="action-cell">
                <label class="inline-check">
                  <input type="checkbox" bind:checked={config.context.enable_recent_context} />
                  <span>{t("useRecentContext")}</span>
                </label>
                <button
                  type="button"
                  class="danger-action"
                  onclick={onClearRecentContext}
                  disabled={clearingRecentContext || (status?.recent_context_count ?? 1) === 0}
                >
                  {clearingRecentContext ? t("privacyClearing") : t("privacyClear")}
                </button>
              </div>
            </td>
          </tr>

          <tr>
            <td><strong>{t("privacyAutoHotwordHistoryData")}</strong></td>
            <td>
              {config.auto_hotwords.enabled ? t("privacySavedWhenEnabled") : t("privacyDisabledNoSave")}
              <small>{entriesCount(status?.auto_hotword_entry_count)}</small>
            </td>
            <td><code>context/hotword_history.jsonl</code></td>
            <td>{t("privacyUploadAutoHotwords")}</td>
            <td>
              <div class="action-cell">
                <label class="inline-check">
                  <input type="checkbox" bind:checked={config.auto_hotwords.enabled} />
                  <span>{t("autoHotwordsEnabled")}</span>
                </label>
                <button
                  type="button"
                  class="danger-action"
                  onclick={onClearAutoHotwordHistory}
                  disabled={clearingAutoHotwordHistory || (status?.auto_hotword_entry_count ?? 1) === 0}
                >
                  {clearingAutoHotwordHistory ? t("privacyClearing") : t("privacyClear")}
                </button>
              </div>
            </td>
          </tr>

          <tr>
            <td><strong>{t("privacyStatsData")}</strong></td>
            <td>
              {t("privacyStatsSaved")}
              <small>{statsCount(status?.stats_event_count)}</small>
            </td>
            <td><code>voice_input_stats.jsonl</code></td>
            <td>{t("privacyUploadNever")}</td>
            <td>
              <button
                type="button"
                class="danger-action"
                onclick={onClearUsageStats}
                disabled={clearingUsageStats || (status?.stats_event_count ?? 1) === 0}
              >
                {clearingUsageStats ? t("privacyClearing") : t("privacyClear")}
              </button>
            </td>
          </tr>

          <tr>
            <td><strong>{t("privacyAsrAudioData")}</strong></td>
            <td>{t("privacyAsrAudioSaved")}</td>
            <td>{t("privacyLocationNotStored")}</td>
            <td>{t("privacyUploadAsrAudio")}</td>
            <td><span class="muted-action">{t("privacyNotApplicable")}</span></td>
          </tr>

          <tr>
            <td><strong>{t("privacyScreenOcrData")}</strong></td>
            <td>{config.screen_context.enabled ? t("privacyRuntimeOnly") : t("privacyDisabledNoSave")}</td>
            <td>{t("privacyLocationNotStored")}</td>
            <td>{t("privacyUploadScreenOcr")}</td>
            <td>
              <label class="inline-check">
                <input type="checkbox" bind:checked={config.screen_context.enabled} />
                <span>{t("enableScreenContext")}</span>
              </label>
            </td>
          </tr>

          <tr>
            <td><strong>{t("privacyLlmTextData")}</strong></td>
            <td>{config.llm_post_edit.enabled ? t("privacyRuntimeOnly") : t("privacyDisabledNoSave")}</td>
            <td>{t("privacyLocationNotStored")}</td>
            <td>{t("privacyUploadLlmText")}</td>
            <td>
              <label class="inline-check">
                <input type="checkbox" bind:checked={config.llm_post_edit.enabled} />
                <span>{t("privacyLlmPolishToggle")}</span>
              </label>
            </td>
          </tr>

          <tr>
            <td><strong>{t("privacyClipboardData")}</strong></td>
            <td>{config.typing.restore_clipboard_after_paste ? t("privacyMemoryOnly") : t("privacyClipboardNoRestore")}</td>
            <td>{t("privacyLocationMemory")}</td>
            <td>{t("privacyUploadNever")}</td>
            <td>
              <label class="inline-check">
                <input type="checkbox" bind:checked={config.typing.restore_clipboard_after_paste} />
                <span>{t("restoreClipboardAfterPaste")}</span>
              </label>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <p class="privacy-note">{t("privacyThirdPartyNotice")}</p>
  </section>
</section>

<style>
  .privacy-stack {
    display: grid;
    gap: 14px;
    max-width: 1120px;
  }

  .privacy-heading {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: end;
    gap: 14px;
    padding: 0 2px;
  }

  .privacy-heading h3,
  .section-heading h3 {
    margin: 0;
    color: var(--text-main);
    font-weight: 800;
  }

  .privacy-heading h3 {
    font-size: 20px;
  }

  .privacy-heading p,
  .section-heading p {
    margin: 4px 0 0;
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.45;
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

  .section-heading {
    display: grid;
    gap: 4px;
  }

  .section-heading h3 {
    font-size: 16px;
  }

  .status-strip {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 10px;
  }

  .status-strip > div {
    display: grid;
    gap: 4px;
    min-width: 0;
    padding: 12px;
    background: #f8fbff;
    border: 1px solid var(--border);
    border-radius: 12px;
  }

  .status-strip span {
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 700;
  }

  .status-strip strong {
    color: var(--text-main);
    font-size: 15px;
    font-weight: 800;
  }

  .table-scroll {
    min-width: 0;
    overflow-x: auto;
    border: 1px solid var(--border);
    border-radius: 12px;
  }

  table {
    width: 100%;
    min-width: 920px;
    border-collapse: collapse;
    background: #ffffff;
  }

  th,
  td {
    padding: 12px;
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border);
    font-size: 13px;
    line-height: 1.45;
    text-align: left;
    vertical-align: top;
  }

  th {
    color: var(--text-main);
    background: #f8fbff;
    font-weight: 800;
  }

  tbody tr:last-child td {
    border-bottom: 0;
  }

  td strong {
    color: var(--text-main);
    font-size: 13px;
    font-weight: 800;
  }

  td small {
    display: block;
    margin-top: 4px;
    color: var(--text-muted);
    font-size: 12px;
  }

  code {
    color: var(--text-main);
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 12px;
    overflow-wrap: anywhere;
  }

  .action-cell {
    display: grid;
    gap: 8px;
    min-width: 0;
  }

  .inline-check {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    color: var(--text-main);
    font-size: 13px;
    font-weight: 700;
    line-height: 1.35;
  }

  .inline-check input {
    width: 17px;
    min-height: 17px;
    flex: 0 0 auto;
    accent-color: var(--primary);
  }

  .inline-check span {
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .secondary-action,
  .danger-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-height: 34px;
    padding: 0 12px;
    border: 1px solid var(--border);
    border-radius: 10px;
    font-size: 13px;
    font-weight: 800;
    line-height: 1.2;
  }

  .secondary-action {
    color: var(--text-main);
    background: #ffffff;
  }

  .danger-action {
    width: fit-content;
    color: #b42318;
    background: #fff7f7;
    border-color: #fecaca;
  }

  .danger-action:disabled {
    cursor: default;
    opacity: 0.55;
  }

  .muted-action {
    color: var(--text-muted);
    font-size: 13px;
    font-weight: 700;
  }

  .privacy-note {
    margin: 0;
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.45;
  }

  @media (max-width: 920px) {
    .privacy-heading,
    .status-strip {
      grid-template-columns: 1fr;
    }

    .secondary-action {
      width: 100%;
    }
  }
</style>
