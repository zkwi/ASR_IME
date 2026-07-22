import { describe, expect, it } from "vitest";
import { wrappedFocusIndex } from "$lib/utils/focusTrap";

describe("focus trap navigation", () => {
  it("wraps Shift+Tab from the first control to the last", () => {
    expect(wrappedFocusIndex(0, -1, 3)).toBe(2);
  });

  it("wraps Tab from the last control to the first", () => {
    expect(wrappedFocusIndex(2, 1, 3)).toBe(0);
  });

  it("keeps a single focusable control selected", () => {
    expect(wrappedFocusIndex(0, 1, 1)).toBe(0);
  });
});
