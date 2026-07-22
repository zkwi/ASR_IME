import { describe, expect, it } from "vitest";
import {
  canEditLoadedConfig,
  configLoadStateForResult,
  configSaveState,
  shouldProtectUnsavedChanges,
} from "$lib/utils/configPersistence";

describe("config persistence state", () => {
  it("treats an existing config as loaded and an absent config as missing", () => {
    expect(configLoadStateForResult({ exists: true })).toBe("loaded");
    expect(configLoadStateForResult({ exists: false })).toBe("missing");
  });

  it("only allows editing after a loaded or explicitly missing result", () => {
    expect(canEditLoadedConfig("not_loaded")).toBe(false);
    expect(canEditLoadedConfig("failed")).toBe(false);
    expect(canEditLoadedConfig("loaded")).toBe(true);
    expect(canEditLoadedConfig("missing")).toBe(true);
  });

  it("keeps a failed save visible while unsaved changes remain", () => {
    expect(configSaveState({ loaded: true, dirty: true, saving: false, savedRecently: false, lastSaveError: "denied" })).toBe("error");
    expect(configSaveState({ loaded: true, dirty: false, saving: false, savedRecently: false, lastSaveError: "denied" })).toBe("idle");
  });

  it("shows saving during a retry and saved only after success", () => {
    expect(configSaveState({ loaded: true, dirty: true, saving: true, savedRecently: false, lastSaveError: "denied" })).toBe("saving");
    expect(configSaveState({ loaded: true, dirty: false, saving: false, savedRecently: true, lastSaveError: "" })).toBe("saved");
  });

  it("protects closing only when dirty changes have failed to save", () => {
    expect(shouldProtectUnsavedChanges(true, "disk is read-only")).toBe(true);
    expect(shouldProtectUnsavedChanges(true, "")).toBe(false);
    expect(shouldProtectUnsavedChanges(false, "disk is read-only")).toBe(false);
  });
});
