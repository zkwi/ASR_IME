<script lang="ts">
  import type {
    AppConfig,
    AudioQualityDiagnostic,
    LastSessionOutcome,
    StatsSnapshot,
    UsageStats,
    UserErrorAction,
  } from "$lib/types/app";
  import ActionPanel from "$lib/components/common/ActionPanel.svelte";
  import type { CopyKey, UserErrorDetail } from "$lib/i18n";
  import { savedHoursForUsage } from "$lib/utils/stats";
  import {
    CalendarDays,
    ChevronRight,
    Clock3,
    Copy,
    Keyboard,
    Mic,
    MousePointerClick,
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
    lastAudioQualityDiagnostic: AudioQualityDiagnostic | null;
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
    onOpenRecordingTroubleshooting: () => void;
    onUserErrorAction: (action: UserErrorAction) => void;
    onCopyLastOutcomeText: (text: string) => Promise<boolean>;
    onToggleRecording: () => void;
    onSelectSection: (section: "Options") => void;
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
    lastAudioQualityDiagnostic,
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
    onOpenRecordingTroubleshooting,
    onUserErrorAction,
    onCopyLastOutcomeText,
    onToggleRecording,
    onSelectSection,
  }: Props = $props();

  const outcomePreviewLimit = 500;
  let lastOutcomeExpanded = $state(false);
  let lastOutcomeCopied = $state(false);
  let copyingLastOutcome = $state(false);
  let lastOutcomeCreatedAt = $state<number | null>(null);
  let visibleAudioQualityDiagnostic = $derived(
    lastAudioQualityDiagnostic?.status === "low_activity" && lastSessionOutcome?.kind === "success"
      ? null
      : lastAudioQualityDiagnostic,
  );

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

  function savedHoursText(usage: UsageStats) {
    return formatHours(savedHoursForUsage(usage, chineseTypingCharsPerMinute)).replace(" h", "");
  }

  function audioQualityTitle(diagnostic: AudioQualityDiagnostic) {
    if (diagnostic.status === "low_volume") return t("audioQualityLowVolumeTitle");
    if (diagnostic.status === "clipping") return t("audioQualityClippingTitle");
    if (diagnostic.status === "low_activity") return t("audioQualityLowActivityTitle");
    return t("audioQualityOkTitle");
  }

  function audioQualityDescription(diagnostic: AudioQualityDiagnostic) {
    if (diagnostic.status === "low_volume") return t("audioQualityLowVolumeDescription");
    if (diagnostic.status === "clipping") return t("audioQualityClippingDescription");
    if (diagnostic.status === "low_activity") return t("audioQualityLowActivityDescription");
    return t("audioQualityOkDescription");
  }

  function formatDbfs(value: number) {
    return `${Math.round(value)} dBFS`;
  }

  function formatDuration(valueMs: number) {
    const seconds = Math.max(0, valueMs / 1000);
    if (seconds < 1) return t("audioQualityDurationSubSecond");
    return t("audioQualityDurationSeconds", { seconds: formatNumber(Math.round(seconds)) });
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
  <div class:listening={recording || sessionBusy} class:error={inputStatus === "error"} class:locked={requiresAsrAuth} class="voice-hero">
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
      <div class="hero-launch">
        <span class="hero-launch-label">{t("desktopControl")}</span>
        <div class="hero-trigger-row">
          <span class:enabled={config.triggers.hotkey_enabled} class="hero-trigger">
            <Keyboard size={15} />
            <span>
              <strong>{formatHotkey(snapshotHotkey)}</strong>
              <small>{config.triggers.hotkey_enabled ? t("mainHotkey") : t("disabled")}</small>
            </span>
          </span>
          <span class:enabled={config.triggers.middle_mouse_enabled} class="hero-trigger">
            <MousePointerClick size={15} />
            <span>
              <strong>{t("middleMouse")}</strong>
              <small>{triggerLabel(config.triggers.middle_mouse_enabled)}</small>
            </span>
          </span>
          <span class:enabled={config.triggers.right_alt_enabled} class="hero-trigger">
            <Keyboard size={15} />
            <span>
              <strong>{t("rightAlt")}</strong>
              <small>{triggerLabel(config.triggers.right_alt_enabled)}</small>
            </span>
          </span>
        </div>
        <button type="button" class="hero-shortcut-button" onclick={() => onSelectSection("Options")}>
          {t("shortcutSettings")} <ChevronRight size={15} />
        </button>
      </div>
    </div>
  </div>
</section>
{#if lastSessionOutcome?.kind === "success"}
  <section class="last-outcome-card success-outcome-card">
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
{:else}
  <section class="last-outcome-card standby-outcome-card">
    <div class="last-outcome-header">
      <div class="last-outcome-copy">
        <strong>{requiresAsrAuth ? t("setupRequired") : inputStatus === "idle" ? t("setupHealthReadyTitle") : inputStatusLabel}</strong>
      </div>
      {#if requiresAsrAuth || inputStatus === "idle"}
        <div class="last-outcome-actions">
          <button type="button" class="link-action compact" onclick={requiresAsrAuth ? onOpenSettings : () => onSelectSection("Options")}>
            {requiresAsrAuth ? t("setupCta") : t("shortcutSettings")}
            <ChevronRight size={14} />
          </button>
        </div>
      {/if}
    </div>
    <p class="last-outcome-description">
      {requiresAsrAuth ? setupRequiredMessage : inputStatus === "idle" ? t("setupHealthReadyDescription", { hotkey: formatHotkey(snapshotHotkey) }) : inputStatusDesc}
    </p>
  </section>
{/if}
{#if visibleAudioQualityDiagnostic}
  <ActionPanel
    title={audioQualityTitle(visibleAudioQualityDiagnostic)}
    description={audioQualityDescription(visibleAudioQualityDiagnostic)}
    meta={t("audioQualityMetrics", {
      rms: formatDbfs(visibleAudioQualityDiagnostic.rms_dbfs),
      peak: formatDbfs(visibleAudioQualityDiagnostic.peak_dbfs),
      active: `${Math.round(visibleAudioQualityDiagnostic.active_ratio * 100)}%`,
      duration: formatDuration(visibleAudioQualityDiagnostic.duration_ms)
    })}
    tone={visibleAudioQualityDiagnostic.status === "ok" ? "info" : "warning"}
    actionVisible={visibleAudioQualityDiagnostic.status !== "ok"}
    ariaLive="polite"
  >
    {#snippet actions()}
      <button type="button" class="link-action compact" onclick={onOpenRecordingTroubleshooting}>
        {t("audioQualityOpenTroubleshooting")} <ChevronRight size={14} />
      </button>
    {/snippet}
  </ActionPanel>
{/if}
<section class="performance-card">
  <div class="section-title-row">
    <h3>{t("recentUsage")}</h3>
  </div>
  <div class="stats-row" aria-label={t("usageSummary")}>
    <article class="stat-card blue">
      <span class="stat-icon"><PenLine size={uiCompact ? 16 : 20} /></span>
      <p>{t("todayInput")}</p>
      <strong>{formatNumber(stats.recent_24h.total_chars)} {t("chars")}</strong>
      <small>{t("savedToday", { hours: savedHoursText(stats.recent_24h) })}</small>
    </article>
    <article class="stat-card purple">
      <span class="stat-icon"><CalendarDays size={uiCompact ? 16 : 20} /></span>
      <p>{t("recent7d")}</p>
      <strong>{formatNumber(stats.recent_7d.total_chars)} {t("chars")}</strong>
      <small>{t("savedToday", { hours: savedHoursText(stats.recent_7d) })}</small>
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
  .performance-card {
    width: 100%;
    margin-inline: 0;
    min-width: 0;
  }

  .voice-card {
    display: grid;
    gap: 12px;
    overflow: visible;
  }

  .last-outcome-card,
  .performance-card {
    padding: 18px;
    overflow: hidden;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 14px;
    box-shadow: var(--shadow-card);
  }

  .performance-card {
    margin-top: 0;
  }

  .last-outcome-card {
    order: 1;
  }

  .performance-card {
    order: 2;
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
    gap: 12px;
    padding: 12px 14px;
    background: #fffbeb;
    border: 1px solid #fde68a;
    border-radius: 12px;
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
    gap: 8px;
  }

  .setup-actions button {
    min-height: 34px;
    padding: 0 12px;
    color: #ffffff;
    background: var(--primary);
    border-radius: 9px;
    font-weight: 600;
  }

  .setup-actions .secondary {
    color: var(--primary);
    background: var(--primary-light);
  }

  .error-help-card {
    display: grid;
    gap: 6px;
    padding: 12px 14px;
    color: #991b1b;
    background: #fff5f5;
    border: 1px solid rgba(239, 68, 68, 0.24);
    border-radius: 12px;
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
    border-radius: 9px;
    font-size: 12px;
    font-weight: 700;
  }

  .error-action-row button:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  .last-outcome-card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 7px;
    background: linear-gradient(180deg, #ffffff 0%, #fbfffd 100%);
    border-color: rgba(16, 185, 129, 0.22);
  }

  .standby-outcome-card {
    background: linear-gradient(180deg, #ffffff 0%, #fbfdff 100%);
    border-color: rgba(47, 128, 237, 0.18);
  }

  .last-outcome-header {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
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
    grid-template-columns: 82px minmax(0, 1fr);
    align-items: center;
    gap: 18px;
    min-height: 132px;
    height: auto;
    padding: 18px 24px;
    overflow: hidden;
    color: #ffffff;
    background: linear-gradient(135deg, #2f80ed 0%, #4f6ff0 100%);
    border: 1px solid rgba(255, 255, 255, 0.22);
    border-radius: 14px;
    box-shadow: 0 14px 30px rgba(47, 128, 237, 0.16);
  }

  .voice-hero.listening {
    background: linear-gradient(135deg, #256fe0 0%, #3f7feb 100%);
    box-shadow: 0 14px 30px rgba(47, 128, 237, 0.18);
  }

  .voice-hero.error {
    background: linear-gradient(135deg, #ef4444 0%, #b91c1c 100%);
    box-shadow: 0 18px 34px rgba(239, 68, 68, 0.16);
  }

  .voice-hero.locked {
    background: linear-gradient(135deg, #4b647f 0%, #4f6ff0 100%);
    box-shadow: 0 14px 30px rgba(71, 85, 105, 0.15);
  }

  .voice-hero::after {
    position: absolute;
    inset: 0;
    content: "";
    background: linear-gradient(118deg, transparent 0%, transparent 64%, rgba(255, 255, 255, 0.09) 64%, rgba(255, 255, 255, 0.035) 74%, transparent 74%);
    pointer-events: none;
  }

  .mic-orb {
    position: relative;
    z-index: 1;
    display: grid;
    width: 78px;
    height: 78px;
    place-items: center;
    color: var(--primary);
    background: rgba(255, 255, 255, 0.18);
    border-radius: 999px;
    transition: transform 160ms ease, opacity 160ms ease, box-shadow 160ms ease;
  }

  .mic-orb:hover {
    transform: translateY(-2px);
  }

  .mic-orb:focus-visible {
    outline: 2px solid rgba(255, 255, 255, 0.72);
    outline-offset: 3px;
  }

  .mic-orb:disabled {
    cursor: not-allowed;
    opacity: 0.72;
    transform: none;
  }

  .mic-ring {
    display: grid;
    width: 60px;
    height: 60px;
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
  }

  .hero-status {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    max-width: 100%;
    margin-bottom: 4px;
    font-size: 21px;
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
    margin: 0 0 5px;
    max-width: 100%;
    font-size: 16px;
    line-height: 1.35;
    overflow-wrap: anywhere;
  }

  .voice-copy p {
    max-width: 100%;
    margin: 0;
    color: rgba(255, 255, 255, 0.88);
    font-size: 13px;
    line-height: 1.34;
    overflow-wrap: anywhere;
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
    border: 1px solid transparent;
    border-radius: 10px;
    font-size: 12px;
    font-weight: 700;
    line-height: 1.2;
    white-space: normal;
    overflow-wrap: anywhere;
    transition: background-color 160ms ease, border-color 160ms ease, transform 160ms ease;
  }

  .link-action:hover:not(:disabled) {
    background: #dfeeff;
    border-color: rgba(47, 128, 237, 0.18);
    transform: translateY(-1px);
  }

  .link-action :global(svg) {
    flex: 0 0 auto;
  }

  .stats-row {
    display: grid;
    gap: 12px;
  }

  .stats-row {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }

  .stat-card {
    position: relative;
    display: grid;
    min-width: 0;
    min-height: 92px;
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 12px;
    transition: border-color 160ms ease, box-shadow 160ms ease, transform 160ms ease;
  }

  .stat-card:hover {
    border-color: rgba(47, 128, 237, 0.2);
    box-shadow: 0 8px 18px rgba(15, 23, 42, 0.05);
    transform: translateY(-1px);
  }

  .hero-launch {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 8px;
    max-width: 100%;
    margin-top: 12px;
  }

  .hero-launch-label {
    min-width: 0;
    color: rgba(255, 255, 255, 0.86);
    font-size: 12px;
    font-weight: 800;
    white-space: nowrap;
  }

  .hero-shortcut-button {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    min-height: 30px;
    padding: 0 10px;
    color: #ffffff;
    background: rgba(255, 255, 255, 0.14);
    border: 1px solid rgba(255, 255, 255, 0.18);
    border-radius: 10px;
    font-size: 12px;
    font-weight: 800;
    line-height: 1.2;
    white-space: nowrap;
    transition: background-color 160ms ease, border-color 160ms ease, transform 160ms ease;
  }

  .hero-shortcut-button:hover {
    background: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.28);
    transform: translateY(-1px);
  }

  .hero-trigger-row {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 6px;
    min-width: 0;
  }

  .hero-trigger {
    display: grid;
    grid-template-columns: 18px minmax(0, 1fr);
    align-items: center;
    gap: 6px;
    min-width: 0;
    min-height: 34px;
    padding: 4px 8px;
    color: rgba(255, 255, 255, 0.74);
    background: rgba(15, 23, 42, 0.12);
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 11px;
  }

  .hero-trigger.enabled {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.18);
    border-color: rgba(255, 255, 255, 0.24);
  }

  .hero-trigger :global(svg) {
    justify-self: center;
    opacity: 0.95;
  }

  .hero-trigger span {
    display: grid;
    min-width: 0;
  }

  .hero-trigger strong,
  .hero-trigger small {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .hero-trigger strong {
    font-size: 12px;
    font-weight: 800;
    line-height: 1.2;
  }

  .hero-trigger small {
    color: rgba(255, 255, 255, 0.68);
    font-size: 10px;
    font-weight: 700;
    line-height: 1.2;
  }

  .hero-trigger.enabled small {
    color: rgba(255, 255, 255, 0.8);
  }

  .stat-card {
    gap: 2px;
    align-content: start;
    min-height: 96px;
    padding: 14px;
  }

  .stat-icon {
    display: grid;
    width: 30px;
    height: 30px;
    place-items: center;
    color: #ffffff;
    border-radius: 10px;
  }

  .stat-card.blue .stat-icon {
    color: #2563eb;
    background: #eff6ff;
  }
  .stat-card.purple .stat-icon {
    color: #7c3aed;
    background: #f5f3ff;
  }
  .stat-card.green .stat-icon {
    color: #059669;
    background: #ecfdf5;
  }
  .stat-card.orange .stat-icon {
    color: #d97706;
    background: #fff7ed;
  }

  .stat-card p {
    margin: 2px 0 0;
    line-height: 1.25;
    color: var(--text-secondary);
    font-size: 13px;
    overflow-wrap: anywhere;
  }

  .stat-card strong {
    display: block;
    margin: 0;
    color: var(--text-main);
    font-size: 18px;
    font-weight: 800;
    line-height: 1.18;
    overflow-wrap: normal;
    word-break: normal;
  }

  .stat-card small {
    display: block;
    margin-top: 1px;
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.35;
    overflow-wrap: anywhere;
  }

  .usage-tip {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    margin: 12px 0 0;
    padding-top: 10px;
    color: var(--text-secondary);
    border-top: 1px solid var(--border);
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

  :global(.ui-compact) .last-outcome-card,
  :global(.ui-compact) .performance-card {
    padding: 14px;
  }

  :global(.ui-compact) .last-outcome-card {
    padding-left: 18px;
  }

  :global(.ui-compact) .section-title-row h3 {
    font-size: 16px;
  }

  :global(.ui-compact) .voice-hero {
    grid-template-columns: 72px minmax(0, 1fr);
    gap: 14px;
    min-height: 124px;
    padding: 16px 20px;
  }

  :global(.ui-compact) .mic-orb {
    width: 68px;
    height: 68px;
  }

  :global(.ui-compact) .mic-ring {
    width: 52px;
    height: 52px;
  }

  :global(.ui-compact) .stats-row {
    gap: 7px;
  }

  :global(.ui-compact) .hero-trigger {
    min-height: 30px;
    padding: 4px 8px;
  }

  :global(.ui-compact) .stat-card {
    min-height: 88px;
    padding: 10px;
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

    .stats-row,
    :global(.ui-compact) .stats-row {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .voice-hero {
      grid-template-columns: 74px minmax(0, 1fr);
      padding: 18px 22px;
    }

    .mic-orb {
      width: 70px;
      height: 70px;
    }

    .mic-ring {
      width: 54px;
      height: 54px;
    }

    .hero-status {
      font-size: 21px;
    }

    .voice-copy h4 {
      font-size: 16px;
    }
  }

  @media (max-width: 640px) {
    .stats-row {
      grid-template-columns: minmax(0, 1fr);
    }

    .voice-hero,
    :global(.ui-compact) .voice-hero {
      grid-template-columns: minmax(0, 1fr);
      justify-items: start;
    }

    .hero-launch {
      grid-template-columns: minmax(0, 1fr);
      justify-items: start;
    }

    .hero-trigger-row {
      width: 100%;
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
