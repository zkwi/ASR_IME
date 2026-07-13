import { describe, expect, it } from "vitest";
import { isBlockingSessionPhase, isQuietAsrWarningCode, sessionPhaseMessageKey } from "./sessionState";

describe("session state utilities", () => {
  it("blocks only transition and output phases", () => {
    expect(isBlockingSessionPhase("starting")).toBe(true);
    expect(isBlockingSessionPhase("waiting_final_result")).toBe(true);
    expect(isBlockingSessionPhase("pasting")).toBe(true);
    expect(isBlockingSessionPhase("recording")).toBe(false);
    expect(isBlockingSessionPhase("idle")).toBe(false);
  });

  it("maps phases to stable copy keys", () => {
    expect(sessionPhaseMessageKey("recording")).toBe("sessionRecording");
    expect(sessionPhaseMessageKey("post_editing")).toBe("sessionPostEditing");
    expect(sessionPhaseMessageKey("failed")).toBe("sessionFailed");
  });

  it("keeps only partial clipboard restore warnings quiet", () => {
    expect(isQuietAsrWarningCode("CLIPBOARD_PARTIAL_RESTORE")).toBe(true);
    expect(isQuietAsrWarningCode("CLIPBOARD_WRITE_FAILED")).toBe(false);
    expect(isQuietAsrWarningCode(null)).toBe(false);
  });
});
