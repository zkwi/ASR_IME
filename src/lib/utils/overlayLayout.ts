import { overlayLineHeight } from "$lib/app/defaults";
import type { OverlayMode } from "$lib/types/app";

const DOUBLE_LINE_MIN_FONT_SIZE = 14;
const DOUBLE_LINE_FONT_SIZE = 18;
const SINGLE_LINE_FONT_SIZE = 20;
const SINGLE_LINE_MIN_FONT_SIZE = 18;
const SINGLE_LINE_MAX_CHARS = 18;
const STRONG_BREAK_CHARS = new Set(["，", "。", "！", "？", "；", "：", ",", ".", "!", "?", ";", ":"]);

export type OverlayDisplayLayout = {
  mode: OverlayMode;
  fontSize: number;
  lineLimit: number;
  lines: string[];
};

export function normalizeOverlayText(text: string) {
  const collapsed = String(text || "").replace(/\s+/g, " ").trim();
  return collapsed ? normalizeOverlayInlineSpacing(collapsed) : "";
}

export function resolveOverlayDisplayText(
  text: string,
  availableHeight: number,
  maxWidth: number,
  measureText: (text: string, fontSize: number) => number,
): OverlayDisplayLayout {
  const singleFont = fontForVisibleLines(1, SINGLE_LINE_FONT_SIZE, SINGLE_LINE_MIN_FONT_SIZE, availableHeight);
  const compactLength = compactTextLength(text);
  const canUseSingleLine =
    !text.includes("\n") &&
    compactLength <= SINGLE_LINE_MAX_CHARS &&
    overlayLinesFitWidth([text], singleFont, maxWidth, measureText);

  if (canUseSingleLine || !canFitOverlayLines(2, availableHeight)) {
    const lines = limitOverlayLines(wrapOverlayText(text, singleFont, maxWidth, measureText), 1);
    return { mode: "single", fontSize: singleFont, lineLimit: 1, lines };
  }

  const doubleFont = Math.min(
    singleFont,
    fontForVisibleLines(2, DOUBLE_LINE_FONT_SIZE, DOUBLE_LINE_MIN_FONT_SIZE, availableHeight),
  );
  const fitted = fitOverlayDisplayText(text, 2, doubleFont, availableHeight, maxWidth, measureText);
  const lineLimit = overlayVisibleLineCount(fitted.lines.length);
  return {
    mode: lineLimit >= 2 ? "double" : "single",
    fontSize: fitted.fontSize,
    lineLimit,
    lines: fitted.lines,
  };
}

export function fontForVisibleLines(lines: number, preferred: number, min: number, availableHeight: number) {
  const fitted = Math.floor((availableHeight - 2) / (lines * overlayLineHeight));
  return Math.max(min, Math.min(preferred, fitted || preferred));
}

export function canFitOverlayLines(lines: number, availableHeight: number, minFontSize = DOUBLE_LINE_MIN_FONT_SIZE) {
  if (lines <= 1) return availableHeight > 0;
  const fitted = Math.floor((availableHeight - 2) / (lines * overlayLineHeight));
  return fitted >= minFontSize;
}

export function rebalanceOverlayDisplayLines(lines: string[], lineLimit: number) {
  if (lineLimit < 2) return lines;
  if (lines.length === 1) return splitOverlayLine(lines[0]);
  return lines;
}

export function fitOverlayDisplayText(
  text: string,
  lineLimit: number,
  preferredFontSize: number,
  availableHeight: number,
  maxWidth: number,
  measureText: (text: string, fontSize: number) => number,
  minFontSize = DOUBLE_LINE_MIN_FONT_SIZE,
) {
  const visibleLines = overlayVisibleLineCount(lineLimit);
  const maxFontSize = fontForVisibleLines(visibleLines, preferredFontSize, minFontSize, availableHeight);
  for (let size = maxFontSize; size >= minFontSize; size -= 1) {
    const lines = limitOverlayLines(
      rebalanceOverlayDisplayLines(
        wrapOverlayText(text, size, maxWidth, measureText),
        lineLimit,
      ),
      visibleLines,
    );
    if (overlayLinesFitWidth(lines, size, maxWidth, measureText)) {
      return { fontSize: size, lines };
    }
  }

  const lines = limitOverlayLines(
    rebalanceOverlayDisplayLines(
      wrapOverlayText(text, minFontSize, maxWidth, measureText),
      lineLimit,
    ),
    visibleLines,
  );
  return { fontSize: minFontSize, lines };
}

export function splitOverlayLine(line: string) {
  const trimmed = String(line || "").trim();
  const chars = Array.from(trimmed);
  const compactLength = Array.from(trimmed.replace(/\s/g, "")).length;
  if (compactLength <= SINGLE_LINE_MAX_CHARS) return [line];
  return splitAtCharIndex(trimmed, Math.ceil(chars.length / 2));
}

function splitAtCharIndex(text: string, splitIndex: number) {
  const chars = Array.from(text);
  const first = chars.slice(0, splitIndex).join("").trimEnd();
  const second = chars.slice(splitIndex).join("").trimStart();
  if (!first || !second) return [text];
  return [first, second];
}

function normalizeOverlayInlineSpacing(line: string) {
  return line
    .replace(/[ \t]+/g, " ")
    .replace(/([\u3400-\u9fff]) ([\u3400-\u9fff])/g, "$1$2")
    .replace(/([，。！？；：、]) /g, "$1")
    .replace(/ ([，。！？；：、])/g, "$1")
    .trim();
}

function compactTextLength(text: string) {
  return Array.from(text.replace(/\s/g, "")).length;
}

function limitOverlayLines(lines: string[], visibleLines: number) {
  const count = overlayVisibleLineCount(visibleLines);
  if (lines.length <= count) return lines;
  return lines.slice(-count);
}

function overlayLinesFitWidth(
  lines: string[],
  fontSize: number,
  maxWidth: number,
  measureText: (text: string, fontSize: number) => number,
) {
  if (!maxWidth) return true;
  return lines.every((line) => measureText(line, fontSize) <= maxWidth + 0.5);
}

export function wrapOverlayText(text: string, fontSize: number, maxWidth: number, measureText: (text: string, fontSize: number) => number) {
  if (!text) return [""];
  if (!maxWidth) return text.split("\n");
  const lines: string[] = [];
  for (const paragraph of text.split("\n")) {
    if (!paragraph) {
      lines.push("");
      continue;
    }
    let current = "";
    for (const char of Array.from(paragraph)) {
      const candidate = current + char;
      if (current && measureText(candidate, fontSize) > maxWidth) {
        if (STRONG_BREAK_CHARS.has(char)) {
          current = candidate;
        } else {
          lines.push(current);
          current = char.trimStart();
        }
      } else {
        current = candidate;
      }
    }
    lines.push(current);
  }
  return lines.length ? lines : [""];
}

export function overlayVisibleLineCount(lineLimit: number) {
  return Math.max(1, Math.min(2, lineLimit || 1));
}

export function overlayAvailableTextHeight(windowHeight: number) {
  return Math.max(1, windowHeight - 24);
}
