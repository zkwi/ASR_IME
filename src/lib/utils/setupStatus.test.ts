import { describe, expect, it } from "vitest";
import { fallbackConfig } from "$lib/app/defaults";
import type { AppConfig } from "$lib/types/app";
import {
  buildLocalSetupStatus,
  currentAsrConnectionStatus,
  mergeSetupStatusFromConfig,
  pasteMethodLabel,
} from "./setupStatus";

function configuredApp(): AppConfig {
  const config = structuredClone(fallbackConfig);
  config.asr.provider = "doubao";
  config.auth.app_key = "test-app";
  config.auth.access_key = "test-access";
  config.triggers.hotkey_enabled = true;
  return config;
}

describe("setup status", () => {
  it("requires auth, an audio device, and at least one trigger", () => {
    const config = configuredApp();
    expect(buildLocalSetupStatus(config)).toMatchObject({ ready: false, missing_auth: false, has_audio_device: false });
    expect(buildLocalSetupStatus(config, [{ index: 0, name: "Microphone", is_default: true }])).toMatchObject({
      ready: true,
      missing_auth: false,
      has_audio_device: true,
    });

    config.triggers.hotkey_enabled = false;
    expect(buildLocalSetupStatus(config, [{ index: 0, name: "Microphone", is_default: true }]).ready).toBe(false);
  });

  it("invalidates ready state when edited config loses authentication", () => {
    const config = configuredApp();
    const current = buildLocalSetupStatus(config, [{ index: 0, name: "Microphone", is_default: true }]);
    config.auth.access_key = "";
    expect(mergeSetupStatusFromConfig(config, current)).toMatchObject({ ready: false, missing_auth: true });
  });

  it("uses only a matching tested ASR fingerprint", () => {
    expect(currentAsrConnectionStatus({
      status: null,
      authReady: true,
      testingAsr: false,
      currentFingerprint: "new",
      testedFingerprint: "old",
      asrConnectionStatus: "tested_ok",
    })).toBe("configured_not_tested");
    expect(currentAsrConnectionStatus({
      status: null,
      authReady: true,
      testingAsr: false,
      currentFingerprint: "same",
      testedFingerprint: "same",
      asrConnectionStatus: "tested_failed",
    })).toBe("tested_failed");
  });

  it("labels paste methods without translation ambiguity", () => {
    const t = (key: string) => key;
    expect(pasteMethodLabel("clipboard_only", t as never)).toBe("clipboardOnly");
    expect(pasteMethodLabel("shift_insert", t as never)).toBe("Shift + Insert");
    expect(pasteMethodLabel("ctrl_v", t as never)).toBe("Ctrl + V");
  });
});
