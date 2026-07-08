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
  overlayVisibleLineCount as getOverlayVisibleLineCount,
  resolveOverlayLayout as getOverlayLayout,
  wrapOverlayText as wrapOverlayTextLines,
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
  let lineLimit = $state(1);
  let displayLines = $state<string[]>([defaultOverlayText]);
  let textElement = $state<HTMLDivElement | null>(null);
  let allLines: string[] = [];
  let scrollOffset = 0;
  let tailHoldSteps = 0;
  let scrollTimer: number | undefined;
  let pollPending = false;
  let configPollPending = false;
  let lastConfigPollAt = 0;
  let smallLayoutLocked = false;
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
    stopScroll();
    applyText(text, true);
  }

  function uiConfigChanged(ui: AppConfig["ui"]) {
    const current = options.getConfig().ui;
    return (
      current.width !== ui.width ||
      current.height !== ui.height ||
      current.margin_bottom !== ui.margin_bottom ||
      current.opacity !== ui.opacity ||
      current.scroll_interval_ms !== ui.scroll_interval_ms ||
      current.background_color !== ui.background_color ||
      current.text_color !== ui.text_color
    );
  }

  function applyText(rawText: string, force = false) {
    const normalized = normalizeOverlayText(rawText) || defaultOverlayText;
    if (!force && normalized === text) return;
    text = normalized;

    if (normalized === defaultOverlayText) {
      smallLayoutLocked = false;
    }

    const availableHeight = availableTextHeight();
    const singleFontSize = 20;
    const singleLineCount = wrapText(normalized, singleFontSize).length;
    const layout = getOverlayLayout(
      normalized,
      smallLayoutLocked,
      availableHeight,
      singleLineCount,
    );

    if (layout.mode === "double" && normalized !== defaultOverlayText) {
      smallLayoutLocked = true;
    }

    mode = layout.mode;
    fontSize = layout.fontSize;
    lineLimit = layout.lineLimit;
    allLines = wrapText(normalized, layout.fontSize);
    const visibleCount = visibleLineCount();
    scrollOffset = Math.max(0, allLines.length - visibleCount);
    tailHoldSteps = allLines.length > visibleCount ? 2 : 1;
    refreshVisibleLines();
  }

  function dispose() {
    stopScroll();
  }

  function wrapText(value: string, size: number) {
    return wrapOverlayTextLines(value, size, textContentWidth(), measureText);
  }

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

  function visibleLineCount() {
    return getOverlayVisibleLineCount(lineLimit);
  }

  function availableTextHeight() {
    if (textElement?.clientHeight) {
      return Math.max(1, textElement.clientHeight);
    }
    return getOverlayAvailableTextHeight(window.innerHeight);
  }

  function refreshVisibleLines() {
    const visibleCount = visibleLineCount();
    if (allLines.length <= visibleCount) {
      stopScroll();
      displayLines = allLines;
      return;
    }

    const end = scrollOffset + visibleCount;
    displayLines = allLines.slice(scrollOffset, end);
    startScroll();
  }

  function startScroll() {
    if (scrollTimer !== undefined) return;
    const intervalMs = Math.max(300, options.getConfig().ui.scroll_interval_ms || 1200);
    scrollTimer = window.setInterval(advanceScroll, intervalMs);
  }

  function stopScroll() {
    if (scrollTimer !== undefined) {
      window.clearInterval(scrollTimer);
      scrollTimer = undefined;
    }
  }

  function advanceScroll() {
    const visibleCount = visibleLineCount();
    if (allLines.length <= visibleCount) {
      stopScroll();
      return;
    }

    if (tailHoldSteps > 0) {
      tailHoldSteps -= 1;
      return;
    }

    if (scrollOffset <= 0) {
      stopScroll();
      return;
    }

    scrollOffset -= 1;
    displayLines = allLines.slice(scrollOffset, scrollOffset + visibleCount);
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
