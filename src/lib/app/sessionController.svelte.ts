import type { CopyKey } from "$lib/i18n";
import type {
  AsrFinalText,
  LastSessionOutcome,
  SessionPhase,
  SessionState,
} from "$lib/types/app";
import {
  isBlockingSessionPhase,
  isQuietAsrWarningCode,
} from "$lib/utils/sessionState";
import { shouldClearSensitivePreviewsForPhase } from "$lib/utils/privacyLifecycle";

type NoticeKind = "success" | "info" | "warning" | "error";

type SafeInvoke = <T>(
  command: string,
  args?: Record<string, unknown>,
  quiet?: boolean,
) => Promise<T | null>;

type SessionControllerOptions = {
  t: (key: CopyKey, values?: Record<string, string>) => string;
  safeInvoke: SafeInvoke;
  notify: (message: string, kind: NoticeKind) => void;
  requireAsrAuthGate: () => boolean;
  userErrorMessage: (code: string | null | undefined, fallback: string) => string;
  shouldOpenSettingsForError: (message: string, code?: string | null) => boolean;
  scrollToSettingsPanel: (panelId: string) => void;
  settingsPanelForError: (message: string, code?: string | null) => string;
  isConfigError: (message: string) => boolean;
  sessionPhaseMessage: (phase: SessionPhase) => string;
  scheduleSucceededIdleHint: () => void;
  getPhase: () => SessionPhase;
  setRecording: (value: boolean) => void;
  setPhase: (value: SessionPhase) => void;
  setErrorCode: (value: string | null | undefined) => void;
  setLastOutcome: (value: LastSessionOutcome) => void;
  setLastAudioQualityDiagnostic: (value: null) => void;
  setAudioLevel: (value: number) => void;
  setStatusMessage: (value: string) => void;
  clearSensitivePreviews: () => void;
};

export function createSessionController(options: SessionControllerOptions) {
  function isBusy(phase = options.getPhase()) {
    return isBlockingSessionPhase(phase);
  }

  async function toggleRecordingFromUi(recording: boolean) {
    if (options.requireAsrAuthGate()) return;
    if (isBusy()) return;
    if (!recording) {
      options.clearSensitivePreviews();
      options.setLastAudioQualityDiagnostic(null);
    }
    const result = await options.safeInvoke<SessionState>("toggle_recording");
    if (result) applyState(result);
  }

  function applyState(state: SessionState) {
    const phase = state.phase ?? (state.recording ? "recording" : "idle");
    options.setRecording(state.recording);
    options.setPhase(phase);
    options.setErrorCode(state.error_code);
    if (shouldClearSensitivePreviewsForPhase(phase)) {
      options.clearSensitivePreviews();
      options.setLastAudioQualityDiagnostic(null);
    }
    if (!state.recording) options.setAudioLevel(0);
    if (state.phase === "failed" && state.error_code) {
      options.setStatusMessage(options.userErrorMessage(state.error_code, state.message));
      if (options.shouldOpenSettingsForError(state.message, state.error_code)) {
        options.scrollToSettingsPanel(options.settingsPanelForError(state.message, state.error_code));
      }
      return;
    }
    if (options.isConfigError(state.message)) {
      options.setStatusMessage(options.userErrorMessage(state.error_code, state.message));
      options.scrollToSettingsPanel(options.settingsPanelForError(state.message, state.error_code));
      return;
    }
    options.setStatusMessage(
      state.phase === "failed" && state.message
        ? options.userErrorMessage(state.error_code, state.message)
        : options.sessionPhaseMessage(phase),
    );
    if (phase === "succeeded") options.scheduleSucceededIdleHint();
  }

  function applyAsrFinalText(payload: AsrFinalText) {
    if (payload.error) {
      options.setErrorCode(payload.error_code);
      const message = options.userErrorMessage(payload.error_code, payload.error);
      options.setStatusMessage(message);
      options.notify(message, "error");
      if (options.shouldOpenSettingsForError(payload.error, payload.error_code)) {
        options.scrollToSettingsPanel(options.settingsPanelForError(payload.error, payload.error_code));
      }
      return;
    }
    const visibleWarning =
      payload.warning && !isQuietAsrWarningCode(payload.warning_code)
        ? payload.warning
        : null;
    if (visibleWarning) {
      options.notify(visibleWarning, "warning");
    }
    options.setLastOutcome({
      kind: "success",
      text: payload.text,
      warning: visibleWarning,
      warningCode: visibleWarning ? payload.warning_code : null,
      createdAt: Date.now(),
    });
    options.setStatusMessage(visibleWarning ?? options.t("sessionSucceeded"));
    if (options.getPhase() === "succeeded") options.scheduleSucceededIdleHint();
  }

  return {
    isBusy,
    toggleRecordingFromUi,
    applyState,
    applyAsrFinalText,
  };
}
