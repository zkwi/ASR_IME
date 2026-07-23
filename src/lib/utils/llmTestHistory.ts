export type LlmTestRecord = {
  succeeded: boolean;
  elapsedMs: number | null;
  testedAt: number;
};

export type LlmTestSummary = {
  sampleCount: number;
  successCount: number;
  successRate: number;
  averageLatencyMs: number | null;
  latestLatencyMs: number | null;
  latestSucceeded: boolean | null;
};

export const LLM_TEST_HISTORY_STORAGE_KEY = "voxtype-llm-test-history-v1";
const maxHistoryRecords = 5;

export function appendLlmTestRecord(
  records: LlmTestRecord[],
  record: LlmTestRecord,
): LlmTestRecord[] {
  return [...records, record].slice(-maxHistoryRecords);
}

export function summarizeLlmTestHistory(records: LlmTestRecord[]): LlmTestSummary {
  const successCount = records.filter((record) => record.succeeded).length;
  const successfulLatencies = records.flatMap((record) =>
    record.succeeded && record.elapsedMs !== null ? [record.elapsedMs] : [],
  );
  const latest = records.at(-1);
  return {
    sampleCount: records.length,
    successCount,
    successRate: records.length > 0 ? Math.round((successCount / records.length) * 100) : 0,
    averageLatencyMs:
      successfulLatencies.length > 0
        ? Math.round(successfulLatencies.reduce((total, value) => total + value, 0) / successfulLatencies.length)
        : null,
    latestLatencyMs: latest?.succeeded ? latest.elapsedMs : null,
    latestSucceeded: latest?.succeeded ?? null,
  };
}

export function parseLlmTestHistory(raw: string | null): LlmTestRecord[] {
  if (!raw) return [];
  try {
    const value: unknown = JSON.parse(raw);
    if (!Array.isArray(value)) return [];
    return value
      .flatMap((item): LlmTestRecord[] => {
        if (!item || typeof item !== "object") return [];
        const candidate = item as Record<string, unknown>;
        if (typeof candidate.succeeded !== "boolean" || !Number.isFinite(candidate.testedAt)) return [];
        const elapsedMs = Number.isFinite(candidate.elapsedMs) && Number(candidate.elapsedMs) >= 0
          ? Number(candidate.elapsedMs)
          : null;
        return [{
          succeeded: candidate.succeeded,
          elapsedMs: candidate.succeeded ? elapsedMs : null,
          testedAt: Number(candidate.testedAt),
        }];
      })
      .slice(-maxHistoryRecords);
  } catch {
    return [];
  }
}

type StorageLike = Pick<Storage, "getItem" | "setItem">;

export function readLlmTestHistory(storage: StorageLike): LlmTestRecord[] {
  try {
    return parseLlmTestHistory(storage.getItem(LLM_TEST_HISTORY_STORAGE_KEY));
  } catch {
    return [];
  }
}

export function saveLlmTestHistory(storage: StorageLike, records: LlmTestRecord[]): void {
  try {
    storage.setItem(LLM_TEST_HISTORY_STORAGE_KEY, JSON.stringify(parseLlmTestHistory(JSON.stringify(records))));
  } catch {
    // 本地存储不可用时保留当前会话数据，不影响连接测试。
  }
}
