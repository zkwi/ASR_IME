import type { CopyKey, Language } from "$lib/i18n";
import type { HistoryDayRow, HistorySummaryCard } from "$lib/components/pages/HistorySection.svelte";
import type { StatsSnapshot, UsageStats } from "$lib/types/app";

type Translate = (key: CopyKey, values?: Record<string, string>) => string;

export function formatSeconds(seconds: number) {
  if (seconds < 60) return `${seconds.toFixed(1)}s`;
  return `${Math.floor(seconds / 60)}m ${Math.round(seconds % 60)}s`;
}

export function formatMinutes(seconds: number, language: Language) {
  const minutes = seconds <= 0 ? 0 : Math.max(1, Math.round(seconds / 60));
  if (language === "en") return `${minutes} min`;
  return `${minutes} ${language === "zh-TW" ? "分鐘" : "分钟"}`;
}

export function formatNumber(value: number, language: Language) {
  return new Intl.NumberFormat(language).format(Math.round(value || 0));
}

export function formatHours(hours: number) {
  if (hours < 0.05) return "0 h";
  return `${hours.toFixed(1)} h`;
}

export function formatSavedHours(hours: number, language: Language) {
  const value = hours < 0.05 ? "0" : hours.toFixed(1);
  if (language === "en") return `${value} h`;
  return `${value} ${language === "zh-TW" ? "小時" : "小时"}`;
}

export function savedHoursForUsage(usage: UsageStats, charsPerMinute: number) {
  const typingHours = usage.total_chars / charsPerMinute / 60;
  const recordingHours = usage.total_seconds / 3600;
  return Math.max(0, typingHours - recordingHours);
}

export function weeklySavedHours(stats: StatsSnapshot, charsPerMinute: number) {
  return savedHoursForUsage(stats.recent_7d, charsPerMinute);
}

export function localDateKey(date: Date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function historySummaryCards(stats: StatsSnapshot, t: Translate, language: Language, charsPerMinute: number): HistorySummaryCard[] {
  const recent24hSaved = savedHoursForUsage(stats.recent_24h, charsPerMinute);
  const weeklySaved = weeklySavedHours(stats, charsPerMinute);
  const numberText = (value: number) => formatNumber(value, language);
  return [
    {
      tone: "blue",
      label: t("todayInput"),
      value: `${numberText(stats.recent_24h.total_chars)} ${t("chars")}`,
      hint: t("savedToday", { hours: formatHours(recent24hSaved).replace(" h", "") }),
    },
    {
      tone: "purple",
      label: t("recent7d"),
      value: `${numberText(stats.recent_7d.total_chars)} ${t("chars")}`,
      hint: t("savedToday", { hours: formatHours(weeklySaved).replace(" h", "") }),
    },
    {
      tone: "green",
      label: t("avgCpm"),
      value: `${stats.recent_7d.avg_chars_per_minute.toFixed(0)} ${t("perMinute")}`,
      hint: t("weeklySavedHoursHint"),
    },
    {
      tone: "orange",
      label: t("savedTime"),
      value: formatSavedHours(weeklySaved, language),
      hint: t("weeklySavedShort"),
    },
  ];
}

export function recentSevenDayDisplayRows(
  stats: StatsSnapshot,
  t: Translate,
  language: Language,
  charsPerMinute: number,
  emptyUsage: () => UsageStats,
): HistoryDayRow[] {
  const byDay = new Map(stats.by_day.map((day) => [day.day, day.stats]));
  const today = new Date();
  return Array.from({ length: 7 }, (_, index) => {
    const date = new Date(today);
    date.setDate(today.getDate() - index);
    const day = localDateKey(date);
    const usage = byDay.get(day) ?? emptyUsage();
    return {
      day,
      chars: `${formatNumber(usage.total_chars, language)} ${t("chars")}`,
      duration: formatMinutes(usage.total_seconds, language),
      speed: `${usage.avg_chars_per_minute.toFixed(0)} ${t("perMinute")}`,
      saved: formatSavedHours(savedHoursForUsage(usage, charsPerMinute), language),
    };
  });
}
