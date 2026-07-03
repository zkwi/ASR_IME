import { listen } from "@tauri-apps/api/event";
import type {
  AsrFinalText,
  AudioDeviceFallbackNotice,
  AudioLevel,
  AudioQualityDiagnostic,
  CloseToTrayRequest,
  OverlayConfig,
  OverlayText,
  SessionState,
  StatsSnapshot,
} from "$lib/types/app";

type NativeUnlistener = Promise<() => void>;

type NativeEventControllerOptions = {
  applySessionState: (state: SessionState) => void;
  applyAsrFinalText: (payload: AsrFinalText) => void;
  applyOverlayText: (payload: OverlayText) => void;
  applyOverlayConfig: (payload: OverlayConfig) => void;
  applyStats: (payload: StatsSnapshot) => void;
  applyAudioLevel: (payload: AudioLevel) => void;
  applyAudioQuality: (payload: AudioQualityDiagnostic) => void;
  handleAudioDeviceFallback: (payload: AudioDeviceFallbackNotice) => void;
  showClosePrompt: (payload: CloseToTrayRequest) => void;
  checkForUpdate: () => void;
};

export function registerNativeEventController(options: NativeEventControllerOptions): NativeUnlistener[] {
  return [
    listen<SessionState>("session-state-changed", (event) => {
      options.applySessionState(event.payload);
    }),
    listen<AsrFinalText>("asr-final-text", (event) => {
      options.applyAsrFinalText(event.payload);
    }),
    listen<OverlayText>("overlay-text", (event) => {
      options.applyOverlayText(event.payload);
    }),
    listen<OverlayConfig>("overlay-config", (event) => {
      options.applyOverlayConfig(event.payload);
    }),
    listen<StatsSnapshot>("usage-stats-updated", (event) => {
      options.applyStats(event.payload);
    }),
    listen<AudioLevel>("audio-level", (event) => {
      options.applyAudioLevel(event.payload);
    }),
    listen<AudioQualityDiagnostic>("audio-quality-diagnostic", (event) => {
      options.applyAudioQuality(event.payload);
    }),
    listen<AudioDeviceFallbackNotice>("audio-device-fallback", (event) => {
      options.handleAudioDeviceFallback(event.payload);
    }),
    listen<CloseToTrayRequest>("close-to-tray-requested", (event) => {
      options.showClosePrompt(event.payload);
    }),
    listen("check-update-requested", () => {
      options.checkForUpdate();
    }),
  ];
}

export function disposeNativeEventController(unlisteners: NativeUnlistener[]) {
  void Promise.all(unlisteners).then((disposers) => {
    for (const dispose of disposers) dispose();
  });
}
