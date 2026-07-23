import { describe, expect, it } from "vitest";
import {
  appendLlmTestRecord,
  parseLlmTestHistory,
  readLlmTestHistory,
  saveLlmTestHistory,
  summarizeLlmTestHistory,
  type LlmTestRecord,
} from "./llmTestHistory";

function record(succeeded: boolean, elapsedMs: number | null, testedAt: number): LlmTestRecord {
  return { succeeded, elapsedMs, testedAt };
}

describe("LLM test history", () => {
  it("keeps only the five most recent test records", () => {
    const records = Array.from({ length: 5 }, (_, index) => record(true, 100 + index, index));

    expect(appendLlmTestRecord(records, record(false, null, 5))).toEqual([
      record(true, 101, 1),
      record(true, 102, 2),
      record(true, 103, 3),
      record(true, 104, 4),
      record(false, null, 5),
    ]);
  });

  it("summarizes success rate and successful latency without counting failures as zero", () => {
    const summary = summarizeLlmTestHistory([
      record(true, 900, 1),
      record(false, null, 2),
      record(true, 1_500, 3),
    ]);

    expect(summary).toEqual({
      sampleCount: 3,
      successCount: 2,
      successRate: 67,
      averageLatencyMs: 1_200,
      latestLatencyMs: 1_500,
      latestSucceeded: true,
    });
  });

  it("counts successful tests even when an older result has no latency", () => {
    expect(summarizeLlmTestHistory([
      record(false, null, 1),
      record(true, null, 2),
    ])).toEqual({
      sampleCount: 2,
      successCount: 1,
      successRate: 50,
      averageLatencyMs: null,
      latestLatencyMs: null,
      latestSucceeded: true,
    });
  });

  it("ignores malformed or privacy-sensitive persisted fields", () => {
    expect(parseLlmTestHistory(JSON.stringify([
      { succeeded: true, elapsedMs: 800, testedAt: 10, apiKey: "do-not-load" },
      { succeeded: "yes", elapsedMs: 200, testedAt: 11 },
      { succeeded: false, elapsedMs: -1, testedAt: 12 },
    ]))).toEqual([
      record(true, 800, 10),
      record(false, null, 12),
    ]);
    expect(parseLlmTestHistory("not-json")).toEqual([]);
  });

  it("reads and saves sanitized history through local storage", () => {
    let stored = JSON.stringify([{ succeeded: true, elapsedMs: 700, testedAt: 1, model: "private" }]);
    const storage = {
      getItem: () => stored,
      setItem: (_key: string, value: string) => {
        stored = value;
      },
    };

    expect(readLlmTestHistory(storage)).toEqual([record(true, 700, 1)]);
    saveLlmTestHistory(storage, [record(false, null, 2)]);
    expect(JSON.parse(stored)).toEqual([record(false, null, 2)]);
  });
});
