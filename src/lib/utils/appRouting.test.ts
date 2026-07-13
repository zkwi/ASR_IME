import { describe, expect, it } from "vitest";
import {
  isConfigError,
  isLlmConfigError,
  sectionForSettingsPanel,
  settingsPanelForError,
  shouldOpenSettingsForError,
} from "./appRouting";

describe("app routing", () => {
  it("routes known setting panels to their sections", () => {
    expect(sectionForSettingsPanel("settings-context")).toBe("Hotwords");
    expect(sectionForSettingsPanel("settings-llm-api")).toBe("ApiConfig");
    expect(sectionForSettingsPanel("settings-audio")).toBe("Options");
  });

  it("classifies ASR and LLM configuration errors", () => {
    expect(isConfigError("豆包 ASR 认证缺失")).toBe(true);
    expect(isConfigError("普通网络错误")).toBe(false);
    expect(isLlmConfigError("thinking strategy is invalid")).toBe(true);
    expect(isLlmConfigError("麦克风不可用")).toBe(false);
  });

  it("opens the most relevant settings panel for an error", () => {
    expect(shouldOpenSettingsForError("", "MIC_DEVICE_NOT_FOUND")).toBe(true);
    expect(settingsPanelForError("", "MIC_DEVICE_NOT_FOUND")).toBe("settings-audio");
    expect(settingsPanelForError("大模型 Base URL 不完整")).toBe("settings-llm-api");
    expect(settingsPanelForError("ASR 未配置")).toBe("settings-auth");
  });
});
