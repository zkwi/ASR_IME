import { describe, expect, it } from "vitest";
import type { StatsSnapshot, UsageStats } from "$lib/types/app";
import {
  formatHours,
  historySummaryCards,
  localDateKey,
  recentSevenDayDisplayRows,
  savedHoursForUsage,
} from "./stats";

function usage(totalChars: number, avgCharsPerMinute: number): UsageStats {
  const totalSeconds = avgCharsPerMinute <= 0 ? 0 : (totalChars / avgCharsPerMinute) * 60;
  return {
    session_count: totalChars > 0 ? 1 : 0,
    total_seconds: totalSeconds,
    total_chars: totalChars,
    total_minutes_int: Math.floor(totalSeconds / 60),
    avg_chars_per_minute: avgCharsPerMinute,
  };
}

function snapshot(recent24h: UsageStats, recent7d: UsageStats): StatsSnapshot {
  return {
    path: "voice_input_stats.jsonl",
    recent_24h: recent24h,
    recent_7d: recent7d,
    by_day: [{ day: localDateKey(new Date()), stats: recent24h }],
    history: [],
  };
}

const t = (key: string, values: Record<string, string> = {}) =>
  key === "savedToday" ? `saved ${values.hours}` : key;

describe("stats utilities", () => {
  it("calculates saved typing time without returning negatives", () => {
    const recent24h = usage(12_147, 180);
    const recent7d = usage(44_432, 180);
    expect(formatHours(savedHoursForUsage(recent24h, 50))).toBe("2.9 h");
    expect(formatHours(recent24h.total_chars / 50 / 60)).toBe("4.0 h");
    expect(formatHours(savedHoursForUsage(recent7d, 50))).toBe("10.7 h");
    expect(savedHoursForUsage({ ...usage(100, 10), total_seconds: 3_600 }, 50)).toBe(0);
  });

  it("builds summary cards and seven-day rows", () => {
    const stats = snapshot(usage(12_147, 180), usage(44_432, 180));
    const cards = historySummaryCards(stats, t as never, "zh-CN", 50);
    expect(cards[0].hint).toBe("saved 2.9");
    expect(cards[1].hint).toBe("saved 10.7");
    expect(cards[2].label).toBe("avgCpm");
    expect(cards[2].hint).toBeUndefined();
    expect(cards[3]).toMatchObject({ label: "savedTime", hint: "weeklySavedHoursHint" });

    const [todayRow] = recentSevenDayDisplayRows(stats, t as never, "zh-CN", 50, () => usage(0, 0));
    expect(todayRow.saved).toBe("2.9 小时");
  });
});
