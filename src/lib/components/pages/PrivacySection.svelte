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
    hasLlmApiConfig: boolean;
    onRefreshStatus: () => void;
    onOpenLlmApiSettings: () => void;
    onOpenRecentContextSettings: () => void;
    onOpenAutoHotwordSettings: () => void;
    onOpenScreenContextSettings: () => void;
    onOpenOutputSettings: () => void;
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
    hasLlmApiConfig,
    onRefreshStatus,
    onOpenLlmApiSettings,
    onOpenRecentContextSettings,
    onOpenAutoHotwordSettings,
    onOpenScreenContextSettings,
    onOpenOutputSettings,
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

  function entriesCharsCount(count: number | undefined, chars: number | undefined) {
    if (count === undefined || chars === undefined) return t("privacyStatusLoading");
    return t("privacyEntriesCharsCount", { count: String(count), chars: String(chars) });
  }

  function statsCount(count: number | undefined) {
    if (count === undefined) return t("privacyStatusLoading");
    return t("privacyStatsCount", { count: String(count) });
  }

  function clearDisabled(isClearing: boolean, count: number | undefined) {
    return isClearing || count === undefined || count === 0;
  }

  function statusText(enabled: boolean) {
    return enabled ? t("privacyStatusEnabled") : t("privacyStatusDisabled");
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
      <h3>{t("privacyDataTableTitle")}</h3>
      <p>{t("privacyLocalDataDescription")}</p>
    </div>

    <div class="status-strip">
      <div>
        <span>{t("privacyRecentContextData")}</span>
        <strong>{entriesCount(status?.recent_context_count)}</strong>
      </div>
      <div>
        <span>{t("privacyAutoHotwordHistoryData")}</span>
        <strong>{entriesCharsCount(status?.auto_hotword_entry_count, status?.auto_hotword_total_chars)}</strong>
      </div>
      <div>
        <span>{t("privacyStatsData")}</span>
        <strong>{statsCount(status?.stats_event_count)}</strong>
      </div>
    </div>

    <div class="table-scroll">
      <table>
        <colgroup>
          <col class="col-type" />
          <col class="col-saved" />
          <col class="col-location" />
          <col class="col-upload" />
          <col class="col-action" />
        </colgroup>
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
          <tr class="group-row">
            <td colspan="5">{t("privacyBaseFilesGroup")}</td>
          </tr>
          <tr>
            <td data-label={t("privacyDataType")}><strong>{t("privacyConfigData")}</strong></td>
            <td data-label={t("privacySavedColumn")}>{t("privacyConfigSaved")}</td>
            <td class="location-cell" data-label={t("privacyLocationColumn")}>
              <span class="location-value">{status?.config_path ?? "config.toml"}</span>
            </td>
            <td data-label={t("privacyUploadColumn")}>{t("privacyUploadConfig")}</td>
            <td data-label={t("privacyActionColumn")}><span class="muted-action">{t("privacyNotApplicable")}</span></td>
          </tr>

          <tr>
            <td data-label={t("privacyDataType")}><strong>{t("privacyLogsDiagnosticsData")}</strong></td>
            <td data-label={t("privacySavedColumn")}>{t("privacyLogsDiagnosticsSaved")}</td>
            <td class="location-cell" data-label={t("privacyLocationColumn")}>
              <span class="location-stack">
                <span class="location-value">{status?.log_path ?? "voice_input.log"}</span>
                <span>{t("privacyLocationLogsDiagnostics")}</span>
              </span>
            </td>
            <td data-label={t("privacyUploadColumn")}>{t("privacyUploadLogsDiagnostics")}</td>
            <td data-label={t("privacyActionColumn")}><span class="muted-action">{t("privacyNotApplicable")}</span></td>
          </tr>

          <tr class="group-row">
            <td colspan="5">{t("privacyLocalRecordsGroup")}</td>
          </tr>
          <tr>
            <td data-label={t("privacyDataType")}><strong>{t("privacyRecentContextData")}</strong></td>
            <td data-label={t("privacySavedColumn")}>
              {config.context.enable_recent_context ? t("privacySavedWhenEnabled") : t("privacyDisabledNoSave")}
              <small>{entriesCount(status?.recent_context_count)}</small>
            </td>
            <td class="location-cell" data-label={t("privacyLocationColumn")}><span class="location-value">context/recent_context.jsonl</span></td>
            <td data-label={t("privacyUploadColumn")}>{t("privacyUploadRecentContext")}</td>
            <td data-label={t("privacyActionColumn")}>
              <div class="action-cell">
                <span class:enabled={config.context.enable_recent_context} class="status-pill">{statusText(config.context.enable_recent_context)}</span>
                <small class="action-hint">
                  {config.llm_post_edit.use_recent_context ? t("privacyLlmRecentContextEnabled") : t("privacyLlmRecentContextDisabled")}
                </small>
                <button type="button" class="secondary-action" onclick={onOpenRecentContextSettings}>{t("privacyManageSettings")}</button>
                <button
                  type="button"
                  class="danger-action"
                  onclick={onClearRecentContext}
                  disabled={clearDisabled(clearingRecentContext, status?.recent_context_count)}
                >
                  {clearingRecentContext ? t("privacyClearing") : t("privacyClear")}
                </button>
              </div>
            </td>
          </tr>

          <tr>
            <td data-label={t("privacyDataType")}><strong>{t("privacyAutoHotwordHistoryData")}</strong></td>
            <td data-label={t("privacySavedColumn")}>
              {config.auto_hotwords.enabled ? t("privacySavedWhenEnabled") : t("privacyDisabledNoSave")}
              <small>{entriesCharsCount(status?.auto_hotword_entry_count, status?.auto_hotword_total_chars)}</small>
            </td>
            <td class="location-cell" data-label={t("privacyLocationColumn")}><span class="location-value">context/hotword_history.jsonl</span></td>
            <td data-label={t("privacyUploadColumn")}>{t("privacyUploadAutoHotwords")}</td>
            <td data-label={t("privacyActionColumn")}>
              <div class="action-cell">
                <span class:enabled={config.auto_hotwords.enabled} class="status-pill">{statusText(config.auto_hotwords.enabled)}</span>
                <button type="button" class="secondary-action" onclick={onOpenAutoHotwordSettings}>{t("privacyManageSettings")}</button>
                <button
                  type="button"
                  class="danger-action"
                  onclick={onClearAutoHotwordHistory}
                  disabled={clearDisabled(clearingAutoHotwordHistory, status?.auto_hotword_entry_count)}
                >
                  {clearingAutoHotwordHistory ? t("privacyClearing") : t("privacyClear")}
                </button>
              </div>
            </td>
          </tr>

          <tr>
            <td data-label={t("privacyDataType")}><strong>{t("privacyStatsData")}</strong></td>
            <td data-label={t("privacySavedColumn")}>
              {t("privacyStatsSaved")}
              <small>{statsCount(status?.stats_event_count)}</small>
            </td>
            <td class="location-cell" data-label={t("privacyLocationColumn")}><span class="location-value">voice_input_stats.jsonl</span></td>
            <td data-label={t("privacyUploadColumn")}>{t("privacyUploadNever")}</td>
            <td data-label={t("privacyActionColumn")}>
              <button
                type="button"
                class="danger-action"
                onclick={onClearUsageStats}
                disabled={clearDisabled(clearingUsageStats, status?.stats_event_count)}
              >
                {clearingUsageStats ? t("privacyClearing") : t("privacyClear")}
              </button>
            </td>
          </tr>

          <tr class="group-row">
            <td colspan="5">{t("privacyRuntimeDataGroup")}</td>
          </tr>
          <tr>
            <td data-label={t("privacyDataType")}><strong>{t("privacyAsrAudioData")}</strong></td>
            <td data-label={t("privacySavedColumn")}>{t("privacyAsrAudioSaved")}</td>
            <td class="location-cell" data-label={t("privacyLocationColumn")}>{t("privacyLocationNotStored")}</td>
            <td data-label={t("privacyUploadColumn")}>{t("privacyUploadAsrAudio")}</td>
            <td data-label={t("privacyActionColumn")}><span class="muted-action">{t("privacyNotApplicable")}</span></td>
          </tr>

          <tr>
            <td data-label={t("privacyDataType")}><strong>{t("privacyScreenOcrData")}</strong></td>
            <td data-label={t("privacySavedColumn")}>{config.screen_context.enabled ? t("privacyRuntimeOnly") : t("privacyDisabledNoSave")}</td>
            <td class="location-cell" data-label={t("privacyLocationColumn")}>{t("privacyLocationNotStored")}</td>
            <td data-label={t("privacyUploadColumn")}>{t("privacyUploadScreenOcr")}</td>
            <td data-label={t("privacyActionColumn")}>
              <div class="action-cell">
                <span class:enabled={config.screen_context.enabled} class="status-pill">{statusText(config.screen_context.enabled)}</span>
                <button type="button" class="secondary-action" onclick={onOpenScreenContextSettings}>{t("privacyManageSettings")}</button>
              </div>
            </td>
          </tr>

          <tr>
            <td data-label={t("privacyDataType")}><strong>{t("privacyLlmTextData")}</strong></td>
            <td data-label={t("privacySavedColumn")}>{config.llm_post_edit.enabled ? t("privacyRuntimeOnly") : t("privacyDisabledNoSave")}</td>
            <td class="location-cell" data-label={t("privacyLocationColumn")}>{t("privacyLocationNotStored")}</td>
            <td data-label={t("privacyUploadColumn")}>{t("privacyUploadLlmText")}</td>
            <td data-label={t("privacyActionColumn")}>
              {#if !config.llm_post_edit.enabled && !hasLlmApiConfig}
                <div class="action-cell">
                  <button type="button" class="secondary-action" onclick={onOpenLlmApiSettings}>{t("goApiConfig")}</button>
                  <small class="action-hint">{t("llmApiRequiredForPolishing")}</small>
                </div>
              {:else}
                <div class="action-cell">
                  <span class:enabled={config.llm_post_edit.enabled} class="status-pill">{statusText(config.llm_post_edit.enabled)}</span>
                  <button type="button" class="secondary-action" onclick={onOpenLlmApiSettings}>{t("privacyManageSettings")}</button>
                </div>
              {/if}
            </td>
          </tr>

          <tr>
            <td data-label={t("privacyDataType")}><strong>{t("privacyClipboardData")}</strong></td>
            <td data-label={t("privacySavedColumn")}>{config.typing.restore_clipboard_after_paste ? t("privacyMemoryOnly") : t("privacyClipboardNoRestore")}</td>
            <td class="location-cell" data-label={t("privacyLocationColumn")}>{t("privacyLocationMemory")}</td>
            <td data-label={t("privacyUploadColumn")}>{t("privacyUploadNever")}</td>
            <td data-label={t("privacyActionColumn")}>
              <div class="action-cell">
                <span class:enabled={config.typing.restore_clipboard_after_paste} class="status-pill">{statusText(config.typing.restore_clipboard_after_paste)}</span>
                <button type="button" class="secondary-action" onclick={onOpenOutputSettings}>{t("privacyManageSettings")}</button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <p class="privacy-note">{t("privacyThirdPartyNotice")}</p>
  </section>
</section>

<style>
  .privacy-heading {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: end;
    gap: 14px;
    padding: 0 2px;
  }

  .privacy-heading h3 {
    margin: 0;
    color: var(--text-main);
    font-weight: 800;
  }

  .privacy-heading h3 {
    font-size: 20px;
  }

  .privacy-heading p {
    margin: 4px 0 0;
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.45;
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
    border-left: 3px solid var(--primary);
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
    min-width: 980px;
    border-collapse: collapse;
    table-layout: fixed;
    background: #ffffff;
  }

  .col-type {
    width: 12%;
  }

  .col-saved,
  .col-action {
    width: 17%;
  }

  .col-location {
    width: 21%;
  }

  .col-upload {
    width: 33%;
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

  tbody tr:not(.group-row):hover td {
    background: #fbfdff;
  }

  .group-row td {
    padding: 9px 12px;
    color: var(--primary);
    background: #f3f8ff;
    border-bottom: 1px solid var(--border);
    font-size: 12px;
    font-weight: 800;
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

  .location-cell {
    color: var(--text-secondary);
    font-weight: 500;
    overflow-wrap: anywhere;
  }

  .location-value {
    color: var(--text-main);
    font-family: inherit;
    font-size: inherit;
    font-weight: 700;
    line-height: inherit;
    overflow-wrap: anywhere;
  }

  .location-stack {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 4px 8px;
    min-width: 0;
  }

  .action-cell {
    display: grid;
    gap: 8px;
    min-width: 0;
  }

  .action-hint {
    margin: 0;
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.35;
  }

  .status-pill {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: fit-content;
    min-height: 26px;
    padding: 0 9px;
    color: var(--text-secondary);
    background: #f1f5f9;
    border: 1px solid var(--border);
    border-radius: 999px;
    font-size: 12px;
    font-weight: 800;
    line-height: 1.2;
  }

  .status-pill.enabled {
    color: #0f6b43;
    background: #ecfdf3;
    border-color: #bbf7d0;
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

  @media (max-width: 1120px) {
    .table-scroll {
      overflow: visible;
      border: 0;
      border-radius: 0;
    }

    table,
    tbody,
    tr,
    td {
      display: block;
      width: 100%;
    }

    table {
      min-width: 0;
      background: transparent;
    }

    thead {
      display: none;
    }

    tbody {
      display: grid;
      gap: 10px;
    }

    tbody tr:not(.group-row) {
      display: grid;
      gap: 9px;
      min-width: 0;
      padding: 12px;
      background: #ffffff;
      border: 1px solid var(--border);
      border-radius: 12px;
    }

    tbody tr:not(.group-row):hover td {
      background: transparent;
    }

    tbody tr:not(.group-row) td {
      display: grid;
      grid-template-columns: 112px minmax(0, 1fr);
      gap: 12px;
      min-width: 0;
      padding: 0;
      border-bottom: 0;
    }

    tbody tr:not(.group-row) td::before {
      content: attr(data-label);
      color: var(--text-muted);
      font-size: 12px;
      font-weight: 800;
      line-height: 1.45;
    }

    tbody tr:not(.group-row) td > small {
      grid-column: 2;
      margin-top: -6px;
    }

    .group-row td {
      border: 1px solid var(--border);
      border-radius: 10px;
    }
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

  @media (max-width: 620px) {
    tbody tr:not(.group-row) td {
      grid-template-columns: 1fr;
      gap: 4px;
    }

    tbody tr:not(.group-row) td > small {
      grid-column: 1;
      margin-top: 0;
    }

    .danger-action {
      width: 100%;
    }
  }
</style>
