import { overlayLineHeight } from "$lib/app/defaults";
import type { OverlayMode } from "$lib/types/app";

const DOUBLE_LINE_MIN_FONT_SIZE = 14;
const DOUBLE_LINE_MAX_FONT_SIZE = 20;
const MIN_SEMANTIC_SPLIT_LENGTH = 10;
const MIN_HARD_SPLIT_LENGTH = 12;
const STRONG_BREAK_CHARS = new Set(["，", "。", "！", "？", "；", "：", ",", ".", "!", "?", ";", ":"]);
const SOFT_BREAK_CHARS = new Set(["、", " ", "\t"]);

export function normalizeOverlayText(text: string) {
  const raw = String(text || "").replace(/\r\n/g, "\n").replace(/\r/g, "\n").trim();
  if (!raw) return "";
  const lines: string[] = [];
  let blankPending = false;
  for (const line of raw.split("\n")) {
    const cleaned = line.trim();
    if (!cleaned) {
      blankPending = lines.length > 0;
      continue;
    }
    if (blankPending) lines.push("");
    lines.push(cleaned);
    blankPending = false;
  }
  return lines.join("\n");
}

export function resolveOverlayLayout(
  text: string,
  forceSmall: boolean,
  availableHeight: number,
  singleWrappedLineCount: number,
): { mode: OverlayMode; fontSize: number; lineLimit: number } {
  const compactLength = Array.from(text.replace(/\s/g, "")).length;
  const singleFont = fontForVisibleLines(1, 20, 18, availableHeight);
  const doubleFont = fontForVisibleLines(2, DOUBLE_LINE_MAX_FONT_SIZE, DOUBLE_LINE_MIN_FONT_SIZE, availableHeight);
  const canSplitSingleLine = canSplitOverlayLine(text);
  if (!forceSmall && singleWrappedLineCount <= 1 && compactLength <= 18 && !canSplitSingleLine) {
    return { mode: "single", fontSize: singleFont, lineLimit: 1 };
  }
  const lineLimit = preferredOverlayLineLimit(
    singleWrappedLineCount,
    availableHeight,
    canSplitSingleLine,
  );
  return {
    mode: lineLimit >= 2 ? "double" : "single",
    fontSize: lineLimit >= 2 ? doubleFont : fontForVisibleLines(1, 18, 14, availableHeight),
    lineLimit,
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

export function preferredOverlayLineLimit(wrappedLineCount: number, availableHeight: number, canSplitSingleLine = false) {
  if (wrappedLineCount <= 1 && !canSplitSingleLine) return 1;
  return canFitOverlayLines(2, availableHeight) ? 2 : 1;
}

export function rebalanceOverlayDisplayLines(lines: string[], lineLimit: number, allowHardSplit = false) {
  if (lineLimit < 2 || lines.length !== 1) return lines;
  return splitOverlayLine(lines[0], allowHardSplit);
}

export function fitOverlayDisplayText(
  text: string,
  lineLimit: number,
  preferredFontSize: number,
  availableHeight: number,
  maxWidth: number,
  measureText: (text: string, fontSize: number) => number,
  allowHardSplit = false,
  minFontSize = DOUBLE_LINE_MIN_FONT_SIZE,
) {
  const visibleLines = overlayVisibleLineCount(lineLimit);
  const maxFontSize = fontForVisibleLines(visibleLines, preferredFontSize, minFontSize, availableHeight);
  for (let size = maxFontSize; size >= minFontSize; size -= 1) {
    const lines = rebalanceOverlayDisplayLines(
      wrapOverlayText(text, size, maxWidth, measureText),
      lineLimit,
      allowHardSplit,
    );
    if (overlayLinesFitWidth(lines, size, maxWidth, measureText)) {
      return { fontSize: size, lines };
    }
  }

  const lines = rebalanceOverlayDisplayLines(
    wrapOverlayText(text, minFontSize, maxWidth, measureText),
    lineLimit,
    allowHardSplit,
  );
  return { fontSize: minFontSize, lines };
}

export function canSplitOverlayLine(line: string, allowHardSplit = false) {
  return splitOverlayLine(line, allowHardSplit).length > 1;
}

export function splitOverlayLine(line: string, allowHardSplit = false) {
  const trimmed = String(line || "").trim();
  const chars = Array.from(trimmed);
  const compactLength = Array.from(trimmed.replace(/\s/g, "")).length;
  if (compactLength < MIN_SEMANTIC_SPLIT_LENGTH) return [line];

  const semanticIndex = bestOverlayBreakIndex(chars);
  if (semanticIndex > 0) {
    return splitAtCharIndex(trimmed, semanticIndex);
  }

  if (!allowHardSplit || compactLength < MIN_HARD_SPLIT_LENGTH) return [line];
  return splitAtCharIndex(trimmed, Math.floor(chars.length / 2));
}

function bestOverlayBreakIndex(chars: string[]) {
  const minLeft = 4;
  const minRight = 3;
  const center = chars.length / 2;
  let best = 0;
  let bestScore = Number.POSITIVE_INFINITY;

  for (let index = 0; index < chars.length; index += 1) {
    const char = chars[index];
    const breakAfter = STRONG_BREAK_CHARS.has(char) || SOFT_BREAK_CHARS.has(char);
    if (!breakAfter) continue;

    const splitIndex = index + 1;
    if (splitIndex < minLeft || chars.length - splitIndex < minRight) continue;
    const score = Math.abs(splitIndex - center) + (SOFT_BREAK_CHARS.has(char) ? 1.5 : 0);
    if (score < bestScore) {
      best = splitIndex;
      bestScore = score;
    }
  }

  return best;
}

function splitAtCharIndex(text: string, splitIndex: number) {
  const chars = Array.from(text);
  const first = chars.slice(0, splitIndex).join("").trimEnd();
  const second = chars.slice(splitIndex).join("").trimStart();
  if (!first || !second) return [text];
  return [first, second];
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
        lines.push(current);
        current = char.trimStart();
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
