export type ConfigLoadState = "not_loaded" | "loaded" | "missing" | "failed";
export type ConfigSaveState = "idle" | "pending" | "saving" | "saved" | "error";

export function configLoadStateForResult(result: { exists: boolean }): ConfigLoadState {
  return result.exists ? "loaded" : "missing";
}

export function canEditLoadedConfig(state: ConfigLoadState) {
  return state === "loaded" || state === "missing";
}

export function configSaveState(params: {
  loaded: boolean;
  dirty: boolean;
  saving: boolean;
  savedRecently: boolean;
  lastSaveError: string;
}): ConfigSaveState {
  if (!params.loaded) return "idle";
  if (params.saving) return "saving";
  if (params.dirty && params.lastSaveError) return "error";
  if (params.dirty) return "pending";
  if (params.savedRecently) return "saved";
  return "idle";
}

export function shouldProtectUnsavedChanges(dirty: boolean, lastSaveError: string) {
  return dirty && Boolean(lastSaveError.trim());
}
