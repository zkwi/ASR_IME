#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import ts from "typescript";

const root = process.cwd();
const statsSource = fs.readFileSync(path.join(root, "src", "lib", "utils", "stats.ts"), "utf8");
const { outputText } = ts.transpileModule(statsSource, {
  compilerOptions: {
    module: ts.ModuleKind.ES2022,
    target: ts.ScriptTarget.ES2022,
  },
});

const stats = await import(`data:text/javascript;base64,${Buffer.from(outputText).toString("base64")}`);

function usage(totalChars, avgCharsPerMinute) {
  const totalSeconds = avgCharsPerMinute <= 0 ? 0 : (totalChars / avgCharsPerMinute) * 60;
  return {
    session_count: totalChars > 0 ? 1 : 0,
    total_seconds: totalSeconds,
    total_chars: totalChars,
    total_minutes_int: Math.floor(totalSeconds / 60),
    avg_chars_per_minute: avgCharsPerMinute,
  };
}

function snapshot(recent24h, recent7d) {
  return {
    path: "voice_input_stats.jsonl",
    recent_24h: recent24h,
    recent_7d: recent7d,
    by_day: [{ day: stats.localDateKey(new Date()), stats: recent24h }],
    history: [],
  };
}

function t(key, values = {}) {
  if (key === "savedToday") return `saved ${values.hours}`;
  return key;
}

const recent24h = usage(12_147, 180);
const recent7d = usage(44_432, 180);
const typingCharsPerMinute = 50;

assert.equal(stats.formatHours(stats.savedHoursForUsage(recent24h, typingCharsPerMinute)), "2.9 h");
assert.equal(stats.formatHours(recent24h.total_chars / typingCharsPerMinute / 60), "4.0 h");
assert.equal(stats.formatHours(stats.savedHoursForUsage(recent7d, typingCharsPerMinute)), "10.7 h");

const slowVoice = {
  ...usage(100, 10),
  total_seconds: 60 * 60,
};
assert.equal(stats.savedHoursForUsage(slowVoice, typingCharsPerMinute), 0);

const statsSnapshot = snapshot(recent24h, recent7d);
const cards = stats.historySummaryCards(statsSnapshot, t, "zh-CN", typingCharsPerMinute);
assert.equal(cards[0].hint, "saved 2.9");
assert.equal(cards[1].hint, "saved 10.7");

const [todayRow] = stats.recentSevenDayDisplayRows(statsSnapshot, t, "zh-CN", typingCharsPerMinute, () => usage(0, 0));
assert.equal(todayRow.saved, "2.9 小时");

console.log("[test-stats] all checks passed");
