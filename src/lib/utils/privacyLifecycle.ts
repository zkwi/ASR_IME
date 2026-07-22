import type { SessionPhase } from "$lib/types/app";

export function shouldClearSensitivePreviewsForPhase(phase: SessionPhase) {
  return phase === "starting" || phase === "recording";
}
