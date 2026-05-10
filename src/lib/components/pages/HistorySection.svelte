<script lang="ts">
  export type HistorySummaryTone = "blue" | "purple" | "green" | "orange";

  export type HistorySummaryCard = {
    tone: HistorySummaryTone;
    label: string;
    value: string;
    hint: string;
  };

  export type HistoryDayRow = {
    day: string;
    chars: string;
    duration: string;
    speed: string;
    saved: string;
  };

  type Props = {
    summaryCards: HistorySummaryCard[];
    dayRows: HistoryDayRow[];
    byDayTitle: string;
    byDayDescription: string;
    dateColumnLabel: string;
    inputCharsLabel: string;
    voiceDurationLabel: string;
    averageSpeedLabel: string;
    savedTimeLabel: string;
  };

  let {
    summaryCards,
    dayRows,
    byDayTitle,
    byDayDescription,
    dateColumnLabel,
    inputCharsLabel,
    voiceDurationLabel,
    averageSpeedLabel,
    savedTimeLabel,
  }: Props = $props();
</script>

<section class="history-page">
  <section class="history-summary">
    {#each summaryCards as card}
      <article class={`history-card ${card.tone}`}>
        <p>{card.label}</p>
        <strong>{card.value}</strong>
        <span>{card.hint}</span>
      </article>
    {/each}
  </section>

  <section class="daily-panel form-panel">
    <div class="section-heading">
      <h3>{byDayTitle}</h3>
      <p>{byDayDescription}</p>
    </div>
    <div class="day-list">
      <div class="day-list-head">
        <span>{dateColumnLabel}</span>
        <span>{inputCharsLabel}</span>
        <span>{voiceDurationLabel}</span>
        <span>{averageSpeedLabel}</span>
        <span>{savedTimeLabel}</span>
      </div>
      {#each dayRows as day}
        <article>
          <span data-label={dateColumnLabel}>{day.day}</span>
          <strong class="metric-cell" data-label={inputCharsLabel}>{day.chars}</strong>
          <span class="duration-cell" data-label={voiceDurationLabel}>{day.duration}</span>
          <span class="metric-cell muted" data-label={averageSpeedLabel}>{day.speed}</span>
          <strong class="metric-cell" data-label={savedTimeLabel}>{day.saved}</strong>
        </article>
      {/each}
    </div>
  </section>
</section>

<style>
  .history-summary {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: 12px;
  }

  .history-card {
    min-width: 0;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 14px;
    box-shadow: 0 8px 18px rgba(15, 23, 42, 0.04);
    transition: border-color 160ms ease, box-shadow 160ms ease, transform 160ms ease;
  }

  .history-card {
    min-height: 100px;
    padding: 16px;
  }

  .history-card:hover {
    border-color: rgba(47, 128, 237, 0.2);
    box-shadow: 0 10px 22px rgba(15, 23, 42, 0.055);
    transform: translateY(-1px);
  }

  .history-card p {
    margin: 0;
    color: #5f7188;
    font-size: 13px;
    font-weight: 700;
    text-transform: none;
  }

  .history-card strong {
    display: block;
    margin-top: 8px;
    color: var(--text-main);
    font-size: clamp(18px, 1.9vw, 21px);
    font-weight: 800;
    line-height: 1.2;
    overflow-wrap: normal;
    white-space: nowrap;
  }

  .history-card span {
    display: block;
    margin-top: 8px;
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.35;
  }

  .history-card.blue {
    border-top: 3px solid var(--primary);
  }

  .history-card.purple {
    border-top: 3px solid var(--gradient-end);
  }

  .history-card.green {
    border-top: 3px solid var(--success);
  }

  .history-card.orange {
    border-top: 3px solid #f97316;
  }

  .daily-panel {
    min-width: 0;
  }

  .day-list {
    display: grid;
    gap: 0;
    min-width: 0;
    overflow: hidden;
  }

  .day-list-head,
  .day-list article {
    display: grid;
    grid-template-columns: minmax(104px, 1.05fr) minmax(92px, 0.9fr) minmax(72px, 0.62fr) minmax(104px, 0.9fr) minmax(84px, 0.76fr);
    align-items: center;
    column-gap: 12px;
    min-height: 44px;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
  }

  .day-list-head {
    min-height: 34px;
    padding-top: 0;
    color: #66788e;
    font-size: 13px;
    font-weight: 700;
  }

  .day-list-head span {
    overflow-wrap: anywhere;
  }

  .day-list article:last-child {
    border-bottom: 0;
  }

  .day-list article {
    transition: background-color 160ms ease;
  }

  .day-list article:hover {
    background: #f8fbff;
  }

  .day-list span {
    min-width: 0;
    color: var(--text-secondary);
    font-size: 13px;
    overflow-wrap: anywhere;
  }

  .day-list strong {
    min-width: 0;
    color: var(--text-main);
    font-size: 14px;
    font-weight: 800;
    overflow-wrap: anywhere;
  }

  .metric-cell {
    font-variant-numeric: tabular-nums;
    font-feature-settings: "tnum";
    text-align: right;
  }

  .metric-cell.muted {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .duration-cell {
    justify-self: end;
    min-width: 0;
    color: var(--text-main);
    font-variant-numeric: tabular-nums;
    font-feature-settings: "tnum";
    font-weight: 700;
    text-align: right;
    overflow-wrap: anywhere;
  }

  .day-list-head span:nth-child(n + 2) {
    text-align: right;
  }

  .day-list article span:first-child {
    text-align: left;
  }

  @media (max-width: 1180px) {
    .day-list-head,
    .day-list article {
      grid-template-columns: minmax(104px, 1fr) minmax(94px, 0.8fr) minmax(82px, 0.62fr) minmax(112px, 0.86fr) minmax(86px, 0.72fr);
      column-gap: 8px;
    }
  }

  @media (max-width: 720px) {
    .day-list {
      gap: 10px;
    }

    .day-list-head {
      display: none;
    }

    .day-list article,
    .day-list article:last-child {
      display: grid;
      grid-template-columns: minmax(0, 1fr);
      gap: 8px;
      min-height: 0;
      padding: 12px;
      background: #ffffff;
      border: 1px solid var(--border);
      border-radius: 12px;
    }

    .day-list article > * {
      display: grid;
      grid-template-columns: 112px minmax(0, 1fr);
      gap: 12px;
      align-items: baseline;
      min-width: 0;
      text-align: left;
    }

    .day-list article > *::before {
      content: attr(data-label);
      color: var(--text-muted);
      font-size: 12px;
      font-weight: 800;
      line-height: 1.45;
    }

    .metric-cell,
    .duration-cell {
      justify-self: stretch;
      text-align: left;
    }
  }

  @media (max-width: 480px) {
    .day-list article > * {
      grid-template-columns: minmax(0, 1fr);
      gap: 2px;
    }
  }
</style>
