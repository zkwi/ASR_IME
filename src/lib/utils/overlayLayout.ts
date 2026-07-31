import { overlayLineHeight } from "$lib/app/defaults";
import type { OverlayMode } from "$lib/types/app";

const DOUBLE_LINE_MIN_FONT_SIZE = 14;
const DOUBLE_LINE_FONT_SIZE = 18;
const SINGLE_LINE_FONT_SIZE = 20;
const SINGLE_LINE_MIN_FONT_SIZE = 18;
const SINGLE_LINE_MAX_CHARS = 18;
// 单行最多占可用宽度的比例。字数上限跟不上窗口宽度，只按字数判断会让塞满整行的字幕仍然只显示一行
const SINGLE_LINE_MAX_FILL = 0.75;
const STRONG_BREAK_CHARS = new Set(["，", "。", "！", "？", "；", "：", ",", ".", "!", "?", ";", ":"]);
const FORBIDDEN_LINE_START_CHARS = new Set([
  ...STRONG_BREAK_CHARS,
  "、",
  "…",
  "—",
  ")",
  "]",
  "}",
  "）",
  "】",
  "》",
  "」",
  "』",
  "”",
  "’",
]);

export type OverlayDisplayLayout = {
  mode: OverlayMode;
  fontSize: number;
  lineLimit: number;
  lines: string[];
};

type OverlayLineCandidate = {
  start: number;
  split: number;
  diff: number;
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
    overlayLinesFitWidth([text], singleFont, maxWidth * SINGLE_LINE_MAX_FILL, measureText);

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
    const lines = layoutOverlayLines(text, visibleLines, size, maxWidth, measureText);
    if (
      overlayLinesFitWidth(lines, size, maxWidth, measureText) &&
      overlayLineStartsAreValid(lines)
    ) {
      return { fontSize: size, lines };
    }
  }

  return {
    fontSize: minFontSize,
    lines: layoutOverlayLines(text, visibleLines, minFontSize, maxWidth, measureText, true),
  };
}

function layoutOverlayLines(
  text: string,
  visibleLines: number,
  fontSize: number,
  maxWidth: number,
  measureText: (text: string, fontSize: number) => number,
  forceTail = false,
) {
  const wrapped = wrapOverlayText(text, fontSize, maxWidth, measureText);
  // 显式换行按原样保留段落结构
  if (visibleLines < 2 || text.includes("\n")) {
    return limitOverlayLines(wrapped, visibleLines);
  }

  const fullTextCandidates = findOverlayLineCandidates(text, fontSize, maxWidth, measureText, false);
  if (fullTextCandidates.best) {
    return fullTextCandidates.best.lines;
  }
  // 完整文本只有以标点开头的切法时，先让字号继续缩小；最小字号仍不行才保留安全尾部
  if (fullTextCandidates.fallback && !forceTail) {
    return fullTextCandidates.fallback.lines;
  }

  // 超长文本直接比较两段真实宽度，保留能完整显示的最长尾部
  const tailCandidates = findOverlayLineCandidates(text, fontSize, maxWidth, measureText, true);
  if (tailCandidates.best) {
    return tailCandidates.best.lines;
  }
  return (tailCandidates.fallback ?? fullTextCandidates.fallback)?.lines ??
    limitOverlayLines(wrapped, visibleLines);
}

/** 把能放进两行的文本按宽度尽量对半拆开，避免贪心换行留下「满行 + 一两个字」的孤字尾行。 */
export function balanceOverlayLines(
  text: string,
  fontSize: number,
  maxWidth: number,
  measureText: (text: string, fontSize: number) => number,
) {
  const trimmed = String(text || "").trim();
  const candidate = findOverlayLineCandidates(trimmed, fontSize, maxWidth, measureText, false).best;
  return candidate?.lines ?? [trimmed];
}

function findOverlayLineCandidates(
  text: string,
  fontSize: number,
  maxWidth: number,
  measureText: (text: string, fontSize: number) => number,
  allowTail: boolean,
) {
  const trimmed = String(text || "").trim();
  const chars = Array.from(trimmed);
  let best: OverlayLineCandidate | null = null;
  let fallback: OverlayLineCandidate | null = null;

  for (let split = 1; split < chars.length; split += 1) {
    const second = chars.slice(split).join("").trimStart();
    const secondWidth = measureText(second, fontSize);
    if (!second || (maxWidth && secondWidth > maxWidth + 0.5)) continue;

    let start = 0;
    let first = chars.slice(0, split).join("").trimEnd();
    let firstWidth = measureText(first, fontSize);
    if (allowTail && maxWidth && firstWidth > maxWidth + 0.5) {
      let earliestFit: { start: number; text: string; width: number } | null = null;
      for (let candidateStart = split - 1; candidateStart >= 0; candidateStart -= 1) {
        const candidateText = chars.slice(candidateStart, split).join("").trim();
        if (!candidateText) continue;
        const candidateWidth = measureText(candidateText, fontSize);
        if (candidateWidth <= maxWidth + 0.5) {
          earliestFit = { start: candidateStart, text: candidateText, width: candidateWidth };
          continue;
        }
        if (earliestFit) break;
      }
      if (!earliestFit) continue;
      ({ start, text: first, width: firstWidth } = earliestFit);
      // 截取尾部时不要人为制造新的标点行首
      while (start > 0 && start < split && FORBIDDEN_LINE_START_CHARS.has(first[0])) {
        start += 1;
        first = chars.slice(start, split).join("").trim();
        firstWidth = measureText(first, fontSize);
      }
    }

    if (!first || !second) continue;
    if (maxWidth && (firstWidth > maxWidth + 0.5 || secondWidth > maxWidth + 0.5)) continue;

    const candidate: OverlayLineCandidate = {
      start,
      split,
      diff: Math.abs(firstWidth - secondWidth),
      lines: [first, second],
    };
    if (FORBIDDEN_LINE_START_CHARS.has(second[0])) {
      fallback = chooseBetterOverlayCandidate(fallback, candidate);
    } else {
      best = chooseBetterOverlayCandidate(best, candidate);
    }
  }

  return { best, fallback };
}

function chooseBetterOverlayCandidate(
  current: OverlayLineCandidate | null,
  candidate: OverlayLineCandidate,
) {
  if (!current || candidate.start < current.start) return candidate;
  if (candidate.start > current.start) return current;
  if (candidate.diff < current.diff) return candidate;
  if (candidate.diff > current.diff) return current;
  // 宽度相同时取靠后的切点，让第一行更长，符合字幕习惯
  return candidate.split >= current.split ? candidate : current;
}

function overlayLineStartsAreValid(lines: string[]) {
  return lines.slice(1).every((line) => !FORBIDDEN_LINE_START_CHARS.has(line[0]));
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
