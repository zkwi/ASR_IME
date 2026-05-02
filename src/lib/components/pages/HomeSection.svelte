<script lang="ts">
  import type { AppConfig, LastSessionOutcome, StatsSnapshot, TriggerKey, UserErrorAction } from "$lib/types/app";
  import type { CopyKey, UserErrorDetail } from "$lib/i18n";
  import {
    CalendarDays,
    Check,
    ChevronRight,
    Clock3,
    Copy,
    Globe2,
    Keyboard,
    MessageSquareText,
    Mic,
    PenLine,
    Sparkles,
    Zap,
  } from "lucide-svelte";

  type Translate = (key: CopyKey, values?: Record<string, string>) => string;
  type InputStatus = "idle" | "listening" | "error";

  type Props = {
    config: AppConfig;
    stats: StatsSnapshot;
    t: Translate;
    uiCompact: boolean;
    recording: boolean;
    saving: boolean;
    inputStatus: InputStatus;
    inputStatusLabel: string;
    inputStatusDesc: string;
    requiresAsrAuth: boolean;
    setupRequiredMessage: string;
    activeErrorDetail: UserErrorDetail | null;
    activeErrorActions: UserErrorAction[];
    lastSessionOutcome: LastSessionOutcome;
    sessionBusy: boolean;
    snapshotHotkey: string;
    chineseTypingCharsPerMinute: number;
    formatHotkey: (value: string) => string;
    formatNumber: (value: number) => string;
    formatHours: (hours: number) => string;
    formatSavedHours: (hours: number) => string;
    weeklySavedHours: () => number;
    usageTipText: () => string;
    triggerLabel: (enabled: boolean) => string;
    onOpenSettings: () => void;
    onOpenSetupGuide: () => void;
    onUserErrorAction: (action: UserErrorAction) => void;
    onCopyLastOutcomeText: (text: string) => Promise<boolean>;
    onToggleRecording: () => void;
    onSelectSection: (section: "Options") => void;
    onToggleTrigger: (key: TriggerKey) => void;
  };

  let {
    config,
    stats,
    t,
    uiCompact,
    recording,
    saving,
    inputStatus,
    inputStatusLabel,
    inputStatusDesc,
    requiresAsrAuth,
    setupRequiredMessage,
    activeErrorDetail,
    activeErrorActions,
    lastSessionOutcome,
    sessionBusy,
    snapshotHotkey,
    chineseTypingCharsPerMinute,
    formatHotkey,
    formatNumber,
    formatHours,
    formatSavedHours,
    weeklySavedHours,
    usageTipText,
    triggerLabel,
    onOpenSettings,
    onOpenSetupGuide,
    onUserErrorAction,
    onCopyLastOutcomeText,
    onToggleRecording,
    onSelectSection,
    onToggleTrigger,
  }: Props = $props();

  const outcomePreviewLimit = 500;
  let lastOutcomeExpanded = $state(false);
  let lastOutcomeCopied = $state(false);
  let copyingLastOutcome = $state(false);
  let lastOutcomeCreatedAt = $state<number | null>(null);

  $effect(() => {
    const createdAt = lastSessionOutcome?.createdAt ?? null;
    if (createdAt !== lastOutcomeCreatedAt) {
      lastOutcomeCreatedAt = createdAt;
      lastOutcomeExpanded = false;
      lastOutcomeCopied = false;
      copyingLastOutcome = false;
    }
  });

  function actionLabel(action: UserErrorAction) {
    switch (action) {
      case "retry_recording":
        return t("errorActionRetry");
      case "open_api_config":
        return t("errorActionConfigure");
      case "open_options":
        return t("errorActionOpenOptions");
      case "open_setup_guide":
        return t("errorActionOpenSetupGuide");
      case "copy_diagnostic_report":
        return t("errorActionCopyDiagnosticReport");
      case "open_log":
        return t("errorActionOpenLogs");
    }
  }

  function outcomeTextPreview(text: string) {
    return text.length > outcomePreviewLimit ? text.slice(0, outcomePreviewLimit) : text;
  }

  async function copyLastOutcome() {
    const outcome = lastSessionOutcome;
    if (!outcome || copyingLastOutcome) return;
    copyingLastOutcome = true;
    try {
      const copied = await onCopyLastOutcomeText(outcome.text);
      if (copied && lastSessionOutcome?.createdAt === outcome.createdAt) {
        lastOutcomeCopied = true;
      }
    } finally {
      copyingLastOutcome = false;
    }
  }
</script>

<section class="voice-card">
  {#if requiresAsrAuth}
    <div class="setup-alert">
      <div>
        <strong>{t("setupRequired")}</strong>
        <p>{setupRequiredMessage}</p>
      </div>
      <div class="setup-actions">
        <button type="button" onclick={onOpenSettings}>{t("setupCta")}</button>
        <button type="button" class="secondary" onclick={onOpenSetupGuide}>{t("setupGuideCta")}</button>
      </div>
    </div>
  {/if}
  {#if inputStatus === "error" && activeErrorDetail}
    <div class="error-help-card">
      <strong>{activeErrorDetail.title}</strong>
      <p><span>{t("errorCauseLabel")}：</span>{activeErrorDetail.cause}</p>
      <p><span>{t("errorActionLabel")}：</span>{activeErrorDetail.action}</p>
      {#if activeErrorActions.length > 0}
        <div class="error-action-row">
          {#each activeErrorActions as action}
            <button
              type="button"
              disabled={action === "retry_recording" && (sessionBusy || requiresAsrAuth)}
              onclick={() => onUserErrorAction(action)}
            >
              {actionLabel(action)}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
  <div class:listening={recording} class:error={inputStatus === "error"} class:locked={requiresAsrAuth} class="voice-hero">
    <button class:listening={recording || sessionBusy} class="mic-orb" aria-label={requiresAsrAuth ? t("authGateTitle") : recording ? t("clickStop") : t("clickStart")} onclick={onToggleRecording} disabled={sessionBusy || requiresAsrAuth}>
      <span class="mic-ring"><Mic size={uiCompact ? 34 : 42} strokeWidth={2.15} /></span>
    </button>
    <div class="voice-copy">
      <div class="hero-status">
        <span class="hero-dot" class:listening={recording} class:error={inputStatus === "error"}></span>
        <strong>{inputStatusLabel}</strong>
      </div>
      <h4>{requiresAsrAuth ? t("authGateTitle") : recording ? t("clickStop") : sessionBusy ? inputStatusLabel : t("clickStart")}</h4>
      <p>{requiresAsrAuth ? t("authGateDescription") : inputStatusDesc}</p>
      <div class="hero-features">
        <span><MessageSquareText size={17} />{t("speakAnywhere")}</span>
        <span><Globe2 size={17} />{t("mixedInput")}</span>
      </div>
    </div>
    <button class="shortcut-help" type="button" onclick={() => onSelectSection("Options")}>
      <Keyboard size={14} />
      <span>{formatHotkey(snapshotHotkey)}</span>
    </button>
  </div>
</section>
{#if lastSessionOutcome?.kind === "success"}
  <section class="last-outcome-card">
    <div class="last-outcome-header">
      <div class="last-outcome-copy">
        <strong>{t("lastOutcomeSuccessTitle")}</strong>
      </div>
      <div class="last-outcome-actions">
        <button type="button" class="link-action compact copy-action" disabled={copyingLastOutcome} onclick={copyLastOutcome}>
          <Copy size={14} />
          {copyingLastOutcome ? t("lastOutcomeCopying") : lastOutcomeCopied ? t("lastOutcomeCopiedShort") : t("lastOutcomeCopyText")}
        </button>
        <button type="button" class="link-action compact" onclick={() => (lastOutcomeExpanded = !lastOutcomeExpanded)}>
          {lastOutcomeExpanded ? t("lastOutcomeHideText") : t("lastOutcomeViewText")}
        </button>
      </div>
    </div>
    <p class="last-outcome-description">{t("lastOutcomeSuccessDescription")}</p>
    {#if lastSessionOutcome.warning}
      <p class="last-outcome-warning">
        <span>{t("lastOutcomeWarningLabel")}：</span>{lastSessionOutcome.warning}
      </p>
    {/if}
    <p class="last-outcome-memory">{t("lastOutcomeTextMemoryHint")}</p>
    {#if lastOutcomeExpanded}
      <div class="last-outcome-text">
        <p>{outcomeTextPreview(lastSessionOutcome.text)}</p>
        {#if lastSessionOutcome.text.length > outcomePreviewLimit}
          <small>{t("lastOutcomeTextTruncated")}</small>
        {/if}
      </div>
    {/if}
  </section>
{/if}
<section class="launch-card">
  <div class="section-title-row">
    <h3>{t("desktopControl")}</h3>
    <button class="link-action" type="button" onclick={() => onSelectSection("Options")}>
      {t("shortcutSettings")} <ChevronRight size={16} />
    </button>
  </div>
  <div class="trigger-grid">
    <label class:active={config.triggers.hotkey_enabled} class:disabled={saving} class="trigger-item">
      <input class="trigger-input" type="checkbox" checked={config.triggers.hotkey_enabled} disabled={saving} onchange={() => onToggleTrigger("hotkey_enabled")} />
      <span class="trigger-check">
        {#if config.triggers.hotkey_enabled}<Check size={uiCompact ? 18 : 24} />{/if}
      </span>
      <div>
        <strong>{formatHotkey(snapshotHotkey)}</strong>
        <p>{config.triggers.hotkey_enabled ? t("mainHotkey") : t("disabled")}</p>
      </div>
    </label>
    <label class:active={config.triggers.middle_mouse_enabled} class:disabled={saving} class="trigger-item">
      <input class="trigger-input" type="checkbox" checked={config.triggers.middle_mouse_enabled} disabled={saving} onchange={() => onToggleTrigger("middle_mouse_enabled")} />
      <span class="trigger-check">
        {#if config.triggers.middle_mouse_enabled}<Check size={uiCompact ? 18 : 24} />{/if}
      </span>
      <div>
        <strong>{t("middleMouse")}</strong>
        <p>{triggerLabel(config.triggers.middle_mouse_enabled)}</p>
      </div>
    </label>
    <label class:active={config.triggers.right_alt_enabled} class:disabled={saving} class="trigger-item">
      <input class="trigger-input" type="checkbox" checked={config.triggers.right_alt_enabled} disabled={saving} onchange={() => onToggleTrigger("right_alt_enabled")} />
      <span class="trigger-check">
        {#if config.triggers.right_alt_enabled}<Check size={uiCompact ? 18 : 24} />{/if}
      </span>
      <div>
        <strong>{t("rightAlt")}</strong>
        <p>{triggerLabel(config.triggers.right_alt_enabled)}</p>
      </div>
    </label>
  </div>
</section>
<section class="performance-card">
  <div class="section-title-row">
    <h3>{t("recentUsage")}</h3>
  </div>
  <div class="stats-row" aria-label={t("usageSummary")}>
    <article class="stat-card blue">
      <span class="stat-icon"><PenLine size={uiCompact ? 16 : 20} /></span>
      <p>{t("todayInput")}</p>
      <strong>{formatNumber(stats.recent_24h.total_chars)} {t("chars")}</strong>
      <small>{t("savedToday", { hours: formatHours(stats.recent_24h.total_chars / chineseTypingCharsPerMinute / 60).replace(" h", "") })}</small>
    </article>
    <article class="stat-card purple">
      <span class="stat-icon"><CalendarDays size={uiCompact ? 16 : 20} /></span>
      <p>{t("recent7d")}</p>
      <strong>{formatNumber(stats.recent_7d.total_chars)} {t("chars")}</strong>
      <small>{t("savedToday", { hours: formatHours(stats.recent_7d.total_chars / chineseTypingCharsPerMinute / 60).replace(" h", "") })}</small>
    </article>
    <article class="stat-card green">
      <span class="stat-icon"><Zap size={uiCompact ? 16 : 20} /></span>
      <p>{t("inputSpeed")}</p>
      <strong>{stats.recent_7d.avg_chars_per_minute.toFixed(0)} {t("perMinute")}</strong>
      <small>{t("avgCpm")}</small>
    </article>
    <article class="stat-card orange">
      <span class="stat-icon"><Clock3 size={uiCompact ? 16 : 20} /></span>
      <p>{t("savedTime")}</p>
      <strong>{formatSavedHours(weeklySavedHours())}</strong>
      <small>{t("weeklySavedShort")}</small>
    </article>
  </div>
  <p class="usage-tip"><Sparkles size={15} />{usageTipText()}</p>
</section>

<style>
  .voice-card,
  .last-outcome-card,
  .launch-card,
  .performance-card {
    min-width: 0;
    padding: 14px;
    overflow: hidden;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 16px;
    box-shadow: var(--shadow-card);
  }

  .launch-card,
  .performance-card {
    margin-top: 0;
  }

  .last-outcome-card {
    order: 1;
  }

  .performance-card {
    order: 2;
  }

  .launch-card {
    order: 3;
  }

  .section-title-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 8px;
    min-width: 0;
  }

  .section-title-row h3 {
    margin: 0;
    min-width: 0;
    color: var(--text-main);
    font-size: 17px;
    font-weight: 800;
    line-height: 1.25;
    overflow-wrap: anywhere;
  }

  .setup-alert {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 16px;
    padding: 14px 16px;
    background: #fff7ed;
    border: 1px solid #fed7aa;
    border-radius: 14px;
  }

  .setup-alert strong {
    color: var(--text-main);
  }

  .setup-alert p {
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

  .error-help-card {
    display: grid;
    gap: 6px;
    margin-bottom: 16px;
    padding: 14px 16px;
    color: #991b1b;
    background: #fff5f5;
    border: 1px solid rgba(239, 68, 68, 0.24);
    border-radius: 14px;
  }

  .error-help-card strong {
    color: #7f1d1d;
    font-size: 15px;
  }

  .error-help-card p {
    margin: 0;
    color: #7f1d1d;
    font-size: 13px;
    line-height: 1.45;
  }

  .error-help-card span {
    font-weight: 800;
  }

  .error-action-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 4px;
  }

  .error-action-row button {
    min-height: 32px;
    padding: 0 10px;
    color: #ffffff;
    background: var(--danger);
    border-radius: 10px;
    font-size: 12px;
    font-weight: 700;
  }

  .error-action-row button:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  .last-outcome-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: #f7fffb;
    border-color: rgba(16, 185, 129, 0.24);
  }

  .last-outcome-header {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: start;
    gap: 10px;
    min-width: 0;
  }

  .last-outcome-actions {
    display: flex;
    flex: 0 0 auto;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 8px;
    min-width: 0;
  }

  .last-outcome-copy {
    display: grid;
    min-width: 0;
  }

  .last-outcome-copy strong {
    color: var(--text-main);
    font-size: 15px;
    font-weight: 800;
  }

  .last-outcome-description {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.4;
    overflow-wrap: anywhere;
  }

  .last-outcome-warning {
    margin: 0;
    color: #92400e !important;
    font-size: 12px;
    line-height: 1.35;
    overflow-wrap: anywhere;
  }

  .last-outcome-warning span {
    font-weight: 800;
  }

  .last-outcome-memory {
    margin: 0;
    color: var(--text-muted) !important;
    font-size: 12px;
    line-height: 1.35;
    overflow-wrap: anywhere;
  }

  .link-action.compact {
    justify-self: end;
  }

  .link-action.copy-action {
    color: #047857;
    background: rgba(16, 185, 129, 0.12);
  }

  .link-action:disabled {
    cursor: wait;
    opacity: 0.64;
  }

  .last-outcome-text {
    display: grid;
    gap: 6px;
    min-width: 0;
    max-height: 108px;
    padding: 10px 12px;
    overflow: auto;
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 12px;
  }

  .last-outcome-text p {
    margin: 0;
    color: var(--text-main);
    font-size: 13px;
    line-height: 1.55;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .last-outcome-text small {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .voice-hero {
    position: relative;
    display: grid;
    grid-template-columns: 116px minmax(0, 1fr);
    align-items: center;
    gap: 24px;
    min-height: 168px;
    height: auto;
    padding: 22px 28px;
    overflow: hidden;
    color: #ffffff;
    background: linear-gradient(135deg, var(--gradient-start) 0%, var(--gradient-end) 100%);
    border-radius: 16px;
    box-shadow: 0 16px 34px rgba(47, 128, 237, 0.2);
  }

  .voice-hero::after {
    position: absolute;
    inset: 0;
    content: "";
    background: linear-gradient(118deg, transparent 0%, transparent 62%, rgba(255, 255, 255, 0.12) 62%, rgba(255, 255, 255, 0.06) 74%, transparent 74%);
    pointer-events: none;
  }

  .mic-orb {
    position: relative;
    z-index: 1;
    display: grid;
    width: 108px;
    height: 108px;
    place-items: center;
    color: var(--primary);
    background: rgba(255, 255, 255, 0.18);
    border-radius: 999px;
    transition: all 160ms ease;
  }

  .mic-orb:hover {
    transform: translateY(-2px);
  }

  .mic-orb:disabled {
    cursor: not-allowed;
    opacity: 0.72;
    transform: none;
  }

  .mic-ring {
    display: grid;
    width: 86px;
    height: 86px;
    place-items: center;
    background: #ffffff;
    border-radius: 999px;
    box-shadow: 0 8px 22px rgba(15, 23, 42, 0.12);
  }

  .mic-orb.listening {
    animation: mic-pulse 1.4s ease-in-out infinite;
  }

  .mic-orb.listening .mic-ring {
    color: var(--danger);
  }

  .voice-copy {
    position: relative;
    z-index: 1;
    min-width: 0;
    padding-right: 140px;
  }

  .hero-status {
    display: inline-flex;
    align-items: flex-start;
    gap: 8px;
    max-width: 100%;
    margin-bottom: 10px;
    font-size: 26px;
    font-weight: 800;
    line-height: 1.16;
  }

  .hero-status strong {
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .hero-dot {
    flex: 0 0 auto;
    width: 11px;
    height: 11px;
    background: #14c38e;
    border-radius: 999px;
  }

  .hero-dot.listening {
    background: #ff5a5f;
    animation: status-blink 1.1s ease-in-out infinite;
  }

  .hero-dot.error {
    background: var(--danger);
  }

  .voice-copy h4 {
    margin: 0 0 8px;
    max-width: 100%;
    font-size: 17px;
    line-height: 1.35;
    overflow-wrap: anywhere;
  }

  .voice-copy p {
    max-width: 100%;
    margin: 0;
    color: rgba(255, 255, 255, 0.88);
    font-size: 14px;
    line-height: 1.45;
    overflow-wrap: anywhere;
  }

  .hero-features {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin-top: 14px;
    overflow: visible;
  }

  .hero-features span {
    display: inline-flex;
    flex: 0 1 auto;
    align-items: center;
    gap: 6px;
    min-width: 0;
    min-height: 30px;
    padding: 6px 10px;
    background: rgba(255, 255, 255, 0.16);
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
    line-height: 1.2;
    overflow-wrap: anywhere;
  }

  .shortcut-help {
    position: absolute;
    right: 18px;
    bottom: 16px;
    z-index: 1;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    max-width: min(150px, 34%);
    min-height: 34px;
    padding: 0 11px;
    color: rgba(255, 255, 255, 0.95);
    background: rgba(255, 255, 255, 0.16);
    border: 1px solid rgba(255, 255, 255, 0.18);
    border-radius: 999px;
    font-size: 12px;
    font-weight: 800;
  }

  .shortcut-help span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .link-action {
    display: inline-flex;
    flex: 0 1 auto;
    align-items: center;
    justify-content: center;
    gap: 4px;
    min-width: 0;
    min-height: 30px;
    padding: 0 10px;
    color: var(--primary);
    background: var(--primary-light);
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
    line-height: 1.2;
    white-space: normal;
    overflow-wrap: anywhere;
  }

  .link-action :global(svg) {
    flex: 0 0 auto;
  }

  .trigger-grid,
  .stats-row {
    display: grid;
    gap: 8px;
  }

  .trigger-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .stats-row {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }

  .trigger-item,
  .stat-card {
    position: relative;
    display: grid;
    min-width: 0;
    min-height: 74px;
    background: #f8fbff;
    border: 1px solid var(--border);
    border-radius: 14px;
  }

  .trigger-item {
    grid-template-columns: 30px minmax(0, 1fr);
    align-items: center;
    gap: 9px;
    padding: 10px;
    cursor: pointer;
  }

  .trigger-item > div {
    min-width: 0;
  }

  .trigger-item.active {
    background: #edf6ff;
    border-color: rgba(47, 128, 237, 0.28);
  }

  .trigger-item.disabled {
    cursor: wait;
    opacity: 0.68;
  }

  .trigger-input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
  }

  .trigger-input:disabled {
    cursor: wait;
  }

  .trigger-check {
    display: grid;
    width: 28px;
    height: 28px;
    place-items: center;
    color: var(--primary);
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 10px;
  }

  .trigger-item strong {
    display: block;
    min-width: 0;
    color: var(--text-main);
    font-size: 14px;
    font-weight: 800;
    line-height: 1.2;
    overflow-wrap: anywhere;
  }

  .trigger-item p {
    margin: 3px 0 0;
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.25;
    overflow-wrap: anywhere;
  }

  .stat-card {
    gap: 2px;
    align-content: start;
    min-height: 82px;
    padding: 9px;
  }

  .stat-icon {
    display: grid;
    width: 28px;
    height: 28px;
    place-items: center;
    color: #ffffff;
    border-radius: 10px;
  }

  .stat-card.blue .stat-icon { background: #2f80ed; }
  .stat-card.purple .stat-icon { background: #7c3aed; }
  .stat-card.green .stat-icon { background: #14c38e; }
  .stat-card.orange .stat-icon { background: #f59e0b; }

  .stat-card p {
    margin: 2px 0 0;
    line-height: 1.25;
    overflow-wrap: anywhere;
  }

  .stat-card strong {
    display: block;
    margin: 0;
    color: var(--text-main);
    font-size: 16px;
    font-weight: 800;
    line-height: 1.18;
    overflow-wrap: normal;
    word-break: normal;
  }

  .stat-card small {
    display: block;
    margin-top: 1px;
    color: var(--text-secondary);
    font-size: 11px;
    line-height: 1.35;
    overflow-wrap: anywhere;
  }

  .usage-tip {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    margin: 8px 0 0;
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.35;
    overflow-wrap: anywhere;
  }

  .usage-tip :global(svg) {
    flex: 0 0 auto;
  }

  @keyframes mic-pulse {
    0%, 100% { box-shadow: 0 0 0 0 rgba(255, 255, 255, 0.18); }
    50% { box-shadow: 0 0 0 16px rgba(255, 255, 255, 0.08); }
  }

  @keyframes status-blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.46; }
  }

  :global(.ui-compact) .voice-card,
  :global(.ui-compact) .last-outcome-card,
  :global(.ui-compact) .launch-card,
  :global(.ui-compact) .performance-card {
    padding: 12px;
  }

  :global(.ui-compact) .section-title-row h3 {
    font-size: 16px;
  }

  :global(.ui-compact) .voice-hero {
    grid-template-columns: 92px minmax(0, 1fr);
    gap: 18px;
    min-height: 148px;
    padding: 18px 22px;
  }

  :global(.ui-compact) .mic-orb {
    width: 88px;
    height: 88px;
  }

  :global(.ui-compact) .mic-ring {
    width: 70px;
    height: 70px;
  }

  :global(.ui-compact) .trigger-grid,
  :global(.ui-compact) .stats-row {
    gap: 7px;
  }

  :global(.ui-compact) .trigger-item {
    min-height: 64px;
    padding: 9px;
  }

  :global(.ui-compact) .stat-card {
    min-height: 78px;
    padding: 9px;
  }

  :global(.ui-compact) .stat-icon {
    width: 26px;
    height: 26px;
  }

  :global(.ui-compact) .stat-card p,
  :global(.ui-compact) .stat-card small,
  :global(.ui-compact) .usage-tip {
    font-size: 11px;
  }

  :global(.ui-compact) .stat-card strong {
    font-size: 15px;
  }

  @media (max-width: 920px) {
    .last-outcome-header {
      grid-template-columns: minmax(0, 1fr);
    }

    .last-outcome-actions {
      justify-content: flex-start;
    }

    .trigger-grid,
    .stats-row,
    :global(.ui-compact) .trigger-grid,
    :global(.ui-compact) .stats-row {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .voice-hero {
      grid-template-columns: 86px minmax(0, 1fr);
      padding: 18px 22px;
    }

    .mic-orb {
      width: 80px;
      height: 80px;
    }

    .mic-ring {
      width: 64px;
      height: 64px;
    }

    .voice-copy {
      padding-right: 106px;
    }

    .hero-status {
      font-size: 21px;
    }

    .voice-copy h4 {
      font-size: 16px;
    }
  }

  @media (max-width: 640px) {
    .trigger-grid,
    .stats-row {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
