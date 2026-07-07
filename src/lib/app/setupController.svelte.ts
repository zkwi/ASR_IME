import type { CopyKey } from "$lib/i18n";
import type {
  AppConfig,
  AsrConnectionStatus,
  AudioDeviceInfo,
  ConnectionTestResult,
  LoadedConfig,
  PersistConfigOptions,
  ScreenContextTestResult,
} from "$lib/types/app";
import { clonePlain } from "$lib/utils/config";
import type { SetupStatus } from "$lib/utils/setupStatus";
import { llmAdapterConfigFingerprint } from "$lib/utils/llmConfig";

type NoticeKind = "success" | "info" | "warning" | "error";

type SafeInvoke = <T>(
  command: string,
  args?: Record<string, unknown>,
  quiet?: boolean,
) => Promise<T | null>;

type SetupControllerOptions = {
  t: (key: CopyKey, values?: Record<string, string>) => string;
  safeInvoke: SafeInvoke;
  notify: (message: string, kind: NoticeKind) => void;
  getConfig: () => AppConfig;
  getStatusMessage: () => string;
  setStatusMessage: (message: string) => void;
  formatNumber: (value: number) => string;
  requireAuthFields: () => boolean;
  persistConfig: (options?: PersistConfigOptions) => Promise<LoadedConfig | null>;
  asrConfigFingerprint: () => string;
  localSetupStatusFromConfig: (config: AppConfig, devices?: AudioDeviceInfo[]) => SetupStatus;
  getSetupStatus: () => SetupStatus | null;
  setSetupStatus: (status: SetupStatus) => void;
  setSetupStatusLoading: (loading: boolean) => void;
  getAudioDevices: () => AudioDeviceInfo[];
  setAudioDevices: (devices: AudioDeviceInfo[]) => void;
  getTestingAsr: () => boolean;
  setTestingAsr: (testing: boolean) => void;
  setAsrConnectionStatus: (status: AsrConnectionStatus) => void;
  setAsrTestedConfigFingerprint: (fingerprint: string) => void;
  getTestingLlm: () => boolean;
  setTestingLlm: (testing: boolean) => void;
  getTestingScreenContext: () => boolean;
  setTestingScreenContext: (testing: boolean) => void;
  setScreenContextTestResult: (result: ScreenContextTestResult | null) => void;
};

type LlmConfigTestOptions = {
  automatic?: boolean;
  expectedFingerprint?: string;
  forceThinkingStrategy?: string;
};

export function createSetupController(options: SetupControllerOptions) {
  async function refreshSetupStatus(showLoading = true) {
    if (showLoading || !options.getSetupStatus()) options.setSetupStatusLoading(true);
    const [devicesResult, setupResult] = await Promise.all([
      options.safeInvoke<AudioDeviceInfo[]>("list_audio_input_devices", undefined, true),
      options.safeInvoke<SetupStatus>("get_setup_status", undefined, true),
    ]);
    if (devicesResult) options.setAudioDevices(devicesResult);
    if (setupResult) {
      options.setSetupStatus(setupResult);
    } else if (!options.getSetupStatus()) {
      options.setSetupStatus(
        options.localSetupStatusFromConfig(options.getConfig(), devicesResult ?? options.getAudioDevices()),
      );
    }
    options.setSetupStatusLoading(false);
  }

  async function testAsrConfig() {
    if (options.getTestingAsr()) return;
    if (!options.requireAuthFields()) return;
    options.setTestingAsr(true);
    options.setAsrConnectionStatus("testing");
    try {
      const result = await options.safeInvoke<ConnectionTestResult>("test_asr_config", {
        config: clonePlain(options.getConfig()),
      });
      if (result) {
        options.setAsrConnectionStatus("tested_ok");
        options.setAsrTestedConfigFingerprint(options.asrConfigFingerprint());
        options.setStatusMessage(options.t("asrTestSucceeded"));
        options.notify(options.getStatusMessage(), "success");
      } else if (options.getStatusMessage()) {
        options.setAsrConnectionStatus("tested_failed");
        options.setAsrTestedConfigFingerprint(options.asrConfigFingerprint());
        options.notify(options.getStatusMessage(), "error");
      }
    } finally {
      options.setTestingAsr(false);
    }
  }

  async function testLlmConfig(testOptions: LlmConfigTestOptions = {}) {
    if (options.getTestingLlm()) return;
    options.setTestingLlm(true);
    try {
      const config = clonePlain(options.getConfig());
      const testedFingerprint = testOptions.expectedFingerprint ?? llmAdapterConfigFingerprint(config);
      if (testOptions.forceThinkingStrategy) {
        config.llm_post_edit.thinking_strategy = testOptions.forceThinkingStrategy;
      }
      if (testOptions.automatic) {
        options.setStatusMessage(options.t("llmAutoTestStarted"));
      }
      const result = await options.safeInvoke<ConnectionTestResult>("test_llm_config", {
        config,
      });
      if (result) {
        const currentConfig = options.getConfig();
        const currentMatchesTestedConfig =
          llmAdapterConfigFingerprint(currentConfig) === testedFingerprint;
        if (testOptions.automatic && !currentMatchesTestedConfig) return;
        let savedStrategy = false;
        if (
          currentMatchesTestedConfig &&
          typeof result.thinking_strategy === "string" &&
          result.thinking_strategy &&
          currentConfig.llm_post_edit.thinking_strategy !== result.thinking_strategy
        ) {
          currentConfig.llm_post_edit.thinking_strategy = result.thinking_strategy;
          savedStrategy = Boolean(await options.persistConfig({ enforceAuth: false, focusErrors: false }));
        }
        const message =
          typeof result.elapsed_ms === "number" && savedStrategy
            ? options.t("llmTestSucceededWithLatencyAndStrategy", {
                ms: options.formatNumber(result.elapsed_ms),
                strategy: result.thinking_strategy ?? "",
              })
            : typeof result.elapsed_ms === "number"
              ? options.t("llmTestSucceededWithLatency", { ms: options.formatNumber(result.elapsed_ms) })
              : options.t("llmTestSucceeded");
        options.setStatusMessage(message);
        options.notify(message, "success");
      } else if (options.getStatusMessage()) {
        options.notify(options.getStatusMessage(), "error");
      }
    } finally {
      options.setTestingLlm(false);
    }
  }

  async function testScreenContext() {
    if (options.getTestingScreenContext()) return;
    options.setTestingScreenContext(true);
    options.setScreenContextTestResult(null);
    try {
      const result = await options.safeInvoke<ScreenContextTestResult>("test_screen_context", {
        config: clonePlain(options.getConfig()),
      });
      if (result) {
        options.setScreenContextTestResult(result);
        const message = options.t("screenContextTestSucceeded", {
          chars: options.formatNumber(result.text_chars),
          ms: options.formatNumber(result.elapsed_ms),
        });
        options.setStatusMessage(message);
        options.notify(message, result.warning ? "warning" : "success");
      } else if (options.getStatusMessage()) {
        options.notify(options.getStatusMessage(), "error");
      }
    } finally {
      options.setTestingScreenContext(false);
    }
  }

  return {
    refreshSetupStatus,
    testAsrConfig,
    testLlmConfig,
    testScreenContext,
  };
}
