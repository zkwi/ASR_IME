import type { CopyKey } from "$lib/i18n";
import type { SessionPhase } from "$lib/types/app";

const blockingSessionPhases = new Set<SessionPhase>([
  "starting",
  "stopping",
  "waiting_final_result",
  "post_editing",
  "pasting",
]);
const sessionStartSourcePhases = new Set<SessionPhase>(["idle", "succeeded", "failed"]);
const sessionEntryPhases = new Set<SessionPhase>(["starting", "recording"]);

export function isBlockingSessionPhase(phase: SessionPhase) {
  return blockingSessionPhases.has(phase);
}

export function startsNewRecordingSession(previousPhase: SessionPhase, nextPhase: SessionPhase) {
  return sessionStartSourcePhases.has(previousPhase) && sessionEntryPhases.has(nextPhase);
}

export function sessionPhaseMessageKey(phase: SessionPhase): CopyKey {
  switch (phase) {
    case "starting":
      return "sessionStarting";
    case "recording":
      return "sessionRecording";
    case "stopping":
      return "sessionStopping";
    case "waiting_final_result":
      return "sessionWaitingFinal";
    case "post_editing":
      return "sessionPostEditing";
    case "pasting":
      return "sessionPasting";
    case "succeeded":
      return "sessionSucceeded";
    case "failed":
      return "sessionFailed";
    case "idle":
    default:
      return "sessionIdleHint";
  }
}

export function isQuietAsrWarningCode(code: string | null | undefined) {
  return code === "CLIPBOARD_PARTIAL_RESTORE";
}
