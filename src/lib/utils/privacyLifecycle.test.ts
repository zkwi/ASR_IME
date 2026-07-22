import { describe, expect, it } from "vitest";

import { shouldClearSensitivePreviewsForPhase } from "./privacyLifecycle";

describe("shouldClearSensitivePreviewsForPhase", () => {
  it("clears previews as a new recording begins", () => {
    expect(shouldClearSensitivePreviewsForPhase("starting")).toBe(true);
    expect(shouldClearSensitivePreviewsForPhase("recording")).toBe(true);
  });

  it("keeps previews for unrelated session states", () => {
    expect(shouldClearSensitivePreviewsForPhase("idle")).toBe(false);
    expect(shouldClearSensitivePreviewsForPhase("succeeded")).toBe(false);
  });
});
