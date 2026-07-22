import { describe, expect, it } from "vitest";
import { invokeErrorCode, overlayStatusText } from "$lib/utils/statusCodes";

describe("localized status codes", () => {
  const t = (key: string) => `translated:${key}`;

  it("localizes known overlay states and keeps fallback text for unknown states", () => {
    expect(overlayStatusText("starting", "fallback", t)).toBe("translated:overlayStatusStarting");
    expect(overlayStatusText("paste_failed", "fallback", t)).toBe("translated:overlayStatusPasteFailed");
    expect(overlayStatusText("future_state", "fallback", t)).toBe("fallback");
  });

  it("uses the fallback when a payload does not carry a status code", () => {
    expect(overlayStatusText(null, "live transcript", t)).toBe("live transcript");
  });

  it("maps update commands to stable frontend error codes", () => {
    expect(invokeErrorCode("check_for_update")).toBe("UPDATE_CHECK_FAILED");
    expect(invokeErrorCode("download_and_install_update")).toBe("UPDATE_DOWNLOAD_FAILED");
    expect(invokeErrorCode("get_usage_stats")).toBeNull();
  });
});
