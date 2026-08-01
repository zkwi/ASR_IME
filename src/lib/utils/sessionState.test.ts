import { describe, expect, it } from "vitest";
import {
  isBlockingSessionPhase,
  isQuietAsrWarningCode,
  sessionPhaseMessageKey,
  startsNewRecordingSession,
} from "./sessionState";

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

  it("counts only terminal-to-recording transitions as a new session", () => {
    expect(startsNewRecordingSession("idle", "starting")).toBe(true);
    expect(startsNewRecordingSession("failed", "recording")).toBe(true);
    expect(startsNewRecordingSession("succeeded", "starting")).toBe(true);

    expect(startsNewRecordingSession("starting", "recording")).toBe(false);
    expect(startsNewRecordingSession("recording", "starting")).toBe(false);
    expect(startsNewRecordingSession("stopping", "recording")).toBe(false);
    expect(startsNewRecordingSession("idle", "idle")).toBe(false);
  });
});
