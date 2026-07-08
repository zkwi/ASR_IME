import { defaultOverlayText } from "$lib/app/defaults";
import type { AppConfig, OverlayConfig, OverlayMode, OverlayText } from "$lib/types/app";
import {
  overlayBackgroundColor as getOverlayBackgroundColor,
  overlayBackgroundRgb as getOverlayBackgroundRgb,
  overlayOpacity as getOverlayOpacity,
  overlayOpacityPresetActive as isOverlayOpacityPresetActive,
  overlayPresetActive as isOverlayPresetActive,
  overlayTextColor as getOverlayTextColor,
} from "$lib/utils/overlayAppearance";
import {
  normalizeOverlayText,
  overlayAvailableTextHeight as getOverlayAvailableTextHeight,
  resolveOverlayDisplayText,
} from "$lib/utils/overlayLayout";

type SafeInvoke = <T>(
  command: string,
  args?: Record<string, unknown>,
  quiet?: boolean,
) => Promise<T | null>;

type OverlayControllerOptions = {
  getConfig: () => AppConfig;
  updateUi: (ui: Partial<AppConfig["ui"]>) => void;
  isOverlay: () => boolean;
  isRecording: () => boolean;
  getAudioLevel: () => number;
  safeInvoke: SafeInvoke;
};

export function createOverlayController(options: OverlayControllerOptions) {
  let measureCanvas: HTMLCanvasElement | undefined;
  let text = $state(defaultOverlayText);
  let mode = $state<OverlayMode>("single");
  let fontSize = $state(20);
  let displayLines = $state<string[]>([defaultOverlayText]);
  let textElement = $state<HTMLDivElement | null>(null);
  let pollPending = false;
  let configPollPending = false;
  let lastConfigPollAt = 0;
  const configPollIntervalMs = 1000;

  async function refreshText() {
    if (pollPending) return;
    pollPending = true;
    try {
      const result = await options.safeInvoke<OverlayText>("get_overlay_text");
      const nextText = result?.text ?? "";
      if (nextText.trim()) applyText(nextText);
    } finally {
      pollPending = false;
    }
  }

  async function refreshConfig(force = false) {
    if (!options.isOverlay() || configPollPending) return;
    const now = Date.now();
    if (!force && now - lastConfigPollAt < configPollIntervalMs) return;
    configPollPending = true;
    lastConfigPollAt = now;
    try {
      const result = await options.safeInvoke<OverlayConfig>("get_overlay_config", undefined, true);
      if (result?.ui) applyConfig(result.ui);
    } finally {
      configPollPending = false;
    }
  }

  function refreshLayout() {
    if (options.isOverlay()) applyText(text, true);
  }

  function applyConfig(ui: AppConfig["ui"]) {
    if (!options.isOverlay()) return;
    if (!uiConfigChanged(ui)) return;
    options.updateUi(ui);
    applyText(text, true);
  }

  function uiConfigChanged(ui: AppConfig["ui"]) {
    const current = options.getConfig().ui;
    return (
      current.width !== ui.width ||
      current.height !== ui.height ||
      current.margin_bottom !== ui.margin_bottom ||
      current.opacity !== ui.opacity ||
      current.background_color !== ui.background_color ||
      current.text_color !== ui.text_color
    );
  }

  function applyText(rawText: string, force = false) {
    const normalized = normalizeOverlayText(rawText) || defaultOverlayText;
    if (!force && normalized === text) return;
    text = normalized;

    const layout = resolveOverlayDisplayText(
      normalized,
      availableTextHeight(),
      textContentWidth(),
      measureText,
    );
    mode = layout.mode;
    fontSize = layout.fontSize;
    displayLines = layout.lines;
  }

  function dispose() {}

  function textContentWidth() {
    if (!textElement) {
      return Math.max(80, window.innerWidth - 88);
    }

    const styles = window.getComputedStyle(textElement);
    const paddingLeft = Number.parseFloat(styles.paddingLeft) || 0;
    const paddingRight = Number.parseFloat(styles.paddingRight) || 0;
    return Math.max(80, textElement.clientWidth - paddingLeft - paddingRight);
  }

  function measureText(value: string, size: number) {
    measureCanvas ??= document.createElement("canvas");
    const context = measureCanvas.getContext("2d");
    if (!context) return Array.from(value).length * size;
    context.font = `400 ${size}px "Microsoft YaHei", "Segoe UI", "PingFang SC", sans-serif`;
    return context.measureText(value).width;
  }

  function availableTextHeight() {
    if (textElement?.clientHeight) {
      return Math.max(1, textElement.clientHeight);
    }
    return getOverlayAvailableTextHeight(window.innerHeight);
  }

  function clampAudioLevel(value: number) {
    if (!Number.isFinite(value)) return 0;
    return Math.max(0, Math.min(1, value));
  }

  function meterLevel() {
    return options.isRecording() ? clampAudioLevel(options.getAudioLevel() * 3.2) : 0;
  }

  function meterBarHeight(index: number) {
    const level = meterLevel();
    const quietHeights = [5, 8, 10, 7];
    const activeHeights = [8, 13, 17, 11];
    const threshold = 0.1 + index * 0.16;
    const target = options.isRecording() && level >= threshold ? activeHeights[index] : quietHeights[index];
    return `${target}px`;
  }

  function meterBarOpacity(index: number) {
    if (!options.isRecording()) return "0.42";
    const level = meterLevel();
    return level >= 0.1 + index * 0.16 ? "0.92" : "0.34";
  }

  function backgroundColor() {
    return getOverlayBackgroundColor(options.getConfig().ui);
  }

  function textColor() {
    return getOverlayTextColor(options.getConfig().ui);
  }

  function backgroundRgb() {
    return getOverlayBackgroundRgb(backgroundColor());
  }

  function opacity() {
    return getOverlayOpacity(options.getConfig().ui);
  }

  function applyOpacity(value: number) {
    options.updateUi({ opacity: value });
  }

  function opacityPresetActive(value: number) {
    return isOverlayOpacityPresetActive(opacity(), value);
  }

  function applyPreset(background: string, textValue: string) {
    options.updateUi({ background_color: background, text_color: textValue });
  }

  function presetActive(background: string, textValue: string) {
    return isOverlayPresetActive(backgroundColor(), textColor(), background, textValue);
  }

  return {
    get mode() { return mode; },
    get fontSize() { return fontSize; },
    get displayLines() { return displayLines; },
    get textElement() { return textElement; },
    set textElement(value: HTMLDivElement | null) {
      textElement = value;
      if (value && options.isOverlay()) applyText(text, true);
    },
    get rootStyle() {
      return `--overlay-bg: ${backgroundColor()}; --overlay-bg-rgb: ${backgroundRgb()}; --overlay-opacity: ${opacity()}; --overlay-text: ${textColor()};`;
    },
    refreshText,
    refreshConfig,
    refreshLayout,
    applyConfig,
    applyText,
    dispose,
    meterBarHeight,
    meterBarOpacity,
    backgroundColor,
    textColor,
    backgroundRgb,
    opacity,
    applyOpacity,
    opacityPresetActive,
    applyPreset,
    presetActive,
  };
}
