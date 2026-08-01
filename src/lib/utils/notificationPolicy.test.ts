import { describe, expect, it } from "vitest";
import { noticeAutoDismissMs, noticeRole } from "$lib/utils/notificationPolicy";

describe("notification policy", () => {
  it("does not hide errors on a wall-clock timer", () => {
    expect(noticeAutoDismissMs("error", false, 12)).toBeNull();
  });

  it("keeps actionable notices visible until the action or close button is used", () => {
    expect(noticeAutoDismissMs("warning", true, 12)).toBeNull();
    expect(noticeAutoDismissMs("success", true, 12)).toBeNull();
  });

  it("auto-dismisses success and info notices while giving warnings longer", () => {
    expect(noticeAutoDismissMs("success", false, 12)).toBe(3200);
    expect(noticeAutoDismissMs("info", false, 12)).toBe(3200);
    expect(noticeAutoDismissMs("warning", false, 12)).toBe(8000);
  });

  it("gives long auto-dismissed messages extra reading time", () => {
    expect(noticeAutoDismissMs("info", false, 81)).toBe(5000);
  });

  it("uses alert semantics only for errors", () => {
    expect(noticeRole("error")).toBe("alert");
    expect(noticeRole("warning")).toBe("status");
  });
});
