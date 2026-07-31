import { overlayLineHeight } from "$lib/app/defaults";
import type { OverlayMode } from "$lib/types/app";

const DOUBLE_LINE_MIN_FONT_SIZE = 14;
const DOUBLE_LINE_FONT_SIZE = 20;
const DOUBLE_LINE_BASE_FONT_SIZE = 18;
const SINGLE_LINE_FONT_SIZE = 20;
const SINGLE_LINE_MIN_FONT_SIZE = 18;
const SINGLE_LINE_COMPACT_FONT_SIZE = 19;
// 双行首行尽量铺满，但尾行至少保留 3 个字符，避免重新出现「满行 + 一两个字」
const DOUBLE_LINE_MIN_TAIL_CHARS = 3;
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
  secondLength: number;
  lines: string[];
};

type OverlayLinePart = {
  text: string;
  width: number;
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
  const preferredSingleFont = fontForVisibleLines(
    1,
    SINGLE_LINE_FONT_SIZE,
    SINGLE_LINE_MIN_FONT_SIZE,
    availableHeight,
  );
  const compactSingleFont = Math.min(preferredSingleFont, SINGLE_LINE_COMPACT_FONT_SIZE);
  const singleFontCandidates = compactSingleFont === preferredSingleFont
    ? [preferredSingleFont]
    : [preferredSingleFont, compactSingleFont];
  const singleFont = text.includes("\n")
    ? null
    : singleFontCandidates.find((size) => overlayLinesFitWidth([text], size, maxWidth, measureText)) ?? null;

  if (singleFont !== null || !canFitOverlayLines(2, availableHeight)) {
    const resolvedSingleFont = singleFont ?? preferredSingleFont;
    const lines = limitOverlayLines(wrapOverlayText(text, resolvedSingleFont, maxWidth, measureText), 1);
    return { mode: "single", fontSize: resolvedSingleFont, lineLimit: 1, lines };
  }

  const maxDoubleFont = Math.min(
    preferredSingleFont,
    fontForVisibleLines(2, DOUBLE_LINE_FONT_SIZE, DOUBLE_LINE_MIN_FONT_SIZE, availableHeight),
  );
  const doubleFont = preferredDoubleLineFont(text, maxDoubleFont, maxWidth, measureText);
  const fitted = fitOverlayDisplayText(text, 2, doubleFont, availableHeight, maxWidth, measureText);
  const lineLimit = overlayVisibleLineCount(fitted.lines.length);
  return {
    mode: lineLimit >= 2 ? "double" : "single",
    fontSize: fitted.fontSize,
    lineLimit,
    lines: fitted.lines,
  };
}

function preferredDoubleLineFont(
  text: string,
  maxFontSize: number,
  maxWidth: number,
  measureText: (text: string, fontSize: number) => number,
) {
  if (!maxWidth || text.includes("\n")) return maxFontSize;
  const baseline = Math.min(maxFontSize, DOUBLE_LINE_BASE_FONT_SIZE);
  for (let size = maxFontSize; size > baseline; size -= 1) {
    if (measureText(text, size) <= maxWidth * 2 + 0.5) return size;
  }
  return baseline;
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

/** 首行尽量利用可用宽度，同时为尾行保留足够内容，避免「机械对半」和「满行 + 孤字」两个极端。 */
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
  const compactLength = Array.from(trimmed.replace(/\s/g, "")).length;
  const minTrailingLineLength = Math.min(DOUBLE_LINE_MIN_TAIL_CHARS, Math.max(1, compactLength - 1));
  let best: OverlayLineCandidate | null = null;
  let fallback: OverlayLineCandidate | null = null;

  const maxMeasuredWidth = maxWidth ? maxWidth + 0.5 : Number.POSITIVE_INFINITY;
  const secondBySplit = new Map<number, OverlayLinePart>();
  let earliestSecondSplit = 1;
  if (maxWidth) {
    earliestSecondSplit = chars.length;
    // 只有靠近结尾、能放进第二行的切点才可能成为候选；避免反复测量整段长文本的后缀
    for (let split = chars.length - 1; split >= 1; split -= 1) {
      const second = chars.slice(split).join("").trimStart();
      if (!second) continue;
      const width = measureText(second, fontSize);
      if (width > maxMeasuredWidth) break;
      earliestSecondSplit = split;
      secondBySplit.set(split, { text: second, width });
    }
    if (!secondBySplit.size) return { best, fallback };
  }

  const firstBySplit = new Map<number, OverlayLinePart>();
  let latestFirstSplit = chars.length - 1;
  if (maxWidth) {
    latestFirstSplit = 0;
    for (let split = 1; split < chars.length; split += 1) {
      const first = chars.slice(0, split).join("").trimEnd();
      if (!first) continue;
      const width = measureText(first, fontSize);
      if (width > maxMeasuredWidth) break;
      latestFirstSplit = split;
      firstBySplit.set(split, { text: first, width });
    }
  }

  const firstCandidateSplit = maxWidth ? earliestSecondSplit : 1;
  const lastCandidateSplit = allowTail ? chars.length - 1 : latestFirstSplit;

  for (let split = firstCandidateSplit; split <= lastCandidateSplit; split += 1) {
    const cachedSecond = secondBySplit.get(split);
    const second = cachedSecond?.text ?? chars.slice(split).join("").trimStart();
    const secondWidth = cachedSecond?.width ?? measureText(second, fontSize);
    if (!second || secondWidth > maxMeasuredWidth) continue;

    let start = 0;
    const cachedFirst = firstBySplit.get(split);
    let first = cachedFirst?.text ?? chars.slice(0, split).join("").trimEnd();
    let firstWidth = cachedFirst?.width ??
      (allowTail && maxWidth && split > latestFirstSplit ? maxMeasuredWidth + 1 : measureText(first, fontSize));
    if (allowTail && maxWidth && firstWidth > maxMeasuredWidth) {
      let earliestFit: { start: number; text: string; width: number } | null = null;
      for (let candidateStart = split - 1; candidateStart >= 0; candidateStart -= 1) {
        const candidateText = chars.slice(candidateStart, split).join("").trim();
        if (!candidateText) continue;
        const candidateWidth = measureText(candidateText, fontSize);
        if (candidateWidth <= maxMeasuredWidth) {
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
    if (firstWidth > maxMeasuredWidth || secondWidth > maxMeasuredWidth) continue;

    const candidate: OverlayLineCandidate = {
      start,
      split,
      diff: Math.abs(firstWidth - secondWidth),
      secondLength: Array.from(second.replace(/\s/g, "")).length,
      lines: [first, second],
    };
    if (FORBIDDEN_LINE_START_CHARS.has(second[0])) {
      fallback = chooseBetterOverlayCandidate(fallback, candidate, allowTail, minTrailingLineLength);
    } else {
      best = chooseBetterOverlayCandidate(best, candidate, allowTail, minTrailingLineLength);
    }
  }

  return { best, fallback };
}

function chooseBetterOverlayCandidate(
  current: OverlayLineCandidate | null,
  candidate: OverlayLineCandidate,
  allowTail: boolean,
  minTrailingLineLength: number,
) {
  if (!current || candidate.start < current.start) return candidate;
  if (candidate.start > current.start) return current;
  if (!allowTail) {
    const currentKeepsTail = current.secondLength >= minTrailingLineLength;
    const candidateKeepsTail = candidate.secondLength >= minTrailingLineLength;
    if (currentKeepsTail !== candidateKeepsTail) {
      return candidateKeepsTail ? candidate : current;
    }
    if (currentKeepsTail && candidate.split !== current.split) {
      return candidate.split > current.split ? candidate : current;
    }
  }
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
