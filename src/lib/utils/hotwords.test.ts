import { describe, expect, it } from "vitest";
import { candidateConfidenceLabel, dedupeHotwords, normalizeHotwords } from "./hotwords";

describe("hotword utilities", () => {
  it("normalizes multiline input and removes blank lines", () => {
    expect(normalizeHotwords(" VoxType \n\n 豆包 ASR  ")).toEqual(["VoxType", "豆包 ASR"]);
  });

  it("deduplicates case-insensitively while preserving first spelling", () => {
    expect(dedupeHotwords(["VoxType", " voxtype ", "豆包", "豆包"])).toEqual(["VoxType", "豆包"]);
  });

  it("clamps candidate confidence to a percentage", () => {
    expect(candidateConfidenceLabel(0.856)).toBe("86%");
    expect(candidateConfidenceLabel(2)).toBe("100%");
    expect(candidateConfidenceLabel(Number.NaN)).toBe("0%");
  });
});
