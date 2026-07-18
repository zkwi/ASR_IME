import { describe, expect, it } from "vitest";
import {
  fitOverlayDisplayText,
  normalizeOverlayText,
  rebalanceOverlayDisplayLines,
  resolveOverlayDisplayText,
  splitOverlayLine,
  wrapOverlayText,
} from "./overlayLayout";

const measureByChar = (text: string, fontSize: number) => Array.from(text).length * fontSize;

describe("overlay layout", () => {
  it("collapses preview line breaks and inline spacing", () => {
    expect(normalizeOverlayText("第一行\r\n第二行")).toBe("第一行第二行");
    expect(normalizeOverlayText("第一行\n\n\t第二行")).toBe("第一行第二行");
    expect(normalizeOverlayText("hello\rworld")).toBe("hello world");
    expect(normalizeOverlayText("领导都这么说了，  要主动拥抱 。")).toBe("领导都这么说了，要主动拥抱。");
  });

  it("splits and rebalances only text longer than the single-line limit", () => {
    expect(splitOverlayLine("还有一个问题是，宝宝整体")).toEqual(["还有一个问题是，宝宝整体"]);
    expect(splitOverlayLine("你好，世界")).toEqual(["你好，世界"]);
    expect(splitOverlayLine("这是一个超过十八个字的实时字幕测试文本")).toEqual([
      "这是一个超过十八个字",
      "的实时字幕测试文本",
    ]);
    expect(rebalanceOverlayDisplayLines(["还有一个问题是，宝宝整体"], 2)).toEqual([
      "还有一个问题是，宝宝整体",
    ]);
    expect(rebalanceOverlayDisplayLines(["这是一个超过十八个字的实时字幕测试文本"], 2)).toEqual([
      "这是一个超过十八个字",
      "的实时字幕测试文本",
    ]);
  });

  it("wraps text by width and preserves explicit newlines", () => {
    expect(wrapOverlayText("一二三四五", 20, 40, measureByChar)).toEqual(["一二", "三四", "五"]);
    expect(wrapOverlayText("第一行\n第二行", 20, 200, measureByChar)).toEqual(["第一行", "第二行"]);
  });

  it("keeps short text on one line", () => {
    expect(resolveOverlayDisplayText("短句", 72, 260, measureByChar)).toEqual({
      mode: "single",
      fontSize: 20,
      lineLimit: 1,
      lines: ["短句"],
    });
    expect(resolveOverlayDisplayText("还有一个问题是，宝宝整体", 72, 260, measureByChar)).toEqual({
      mode: "single",
      fontSize: 20,
      lineLimit: 1,
      lines: ["还有一个问题是，宝宝整体"],
    });
  });

  it("selects line count after preview whitespace normalization", () => {
    const shortText = normalizeOverlayText("短句\n测试");
    expect(resolveOverlayDisplayText(shortText, 72, 260, measureByChar)).toEqual({
      mode: "single",
      fontSize: 20,
      lineLimit: 1,
      lines: ["短句测试"],
    });

    const longText = normalizeOverlayText(
      "这是一段超过十八个字的实时字幕\n\n用于验证连续换行不会占用显示行",
    );
    const layout = resolveOverlayDisplayText(longText, 72, 260, measureByChar);
    expect(layout).toMatchObject({ mode: "double", lineLimit: 2 });
    expect(layout.lines).toHaveLength(2);
    expect(layout.lines.every((line) => line.length > 0)).toBe(true);
  });

  it("uses at most two fitted lines for longer text", () => {
    const shortLayout = resolveOverlayDisplayText("短句", 72, 260, measureByChar);
    const doubleLayout = resolveOverlayDisplayText(
      "这是一个超过十八个字的实时字幕测试文本",
      72,
      400,
      measureByChar,
    );
    expect(doubleLayout.mode).toBe("double");
    expect(doubleLayout.lineLimit).toBe(2);
    expect(doubleLayout.fontSize).toBeLessThanOrEqual(shortLayout.fontSize);
    expect(doubleLayout.lines).toEqual(["这是一个超过十八个字", "的实时字幕测试文本"]);

    const longLayout = resolveOverlayDisplayText(
      "这是一段更长的实时字幕测试文本，用来模拟较长口述时的两行显示效果",
      72,
      260,
      measureByChar,
    );
    expect(longLayout).toMatchObject({ mode: "double", fontSize: 18, lineLimit: 2 });
    expect(longLayout.lines).toHaveLength(2);
    expect(longLayout.lines.every((line) => measureByChar(line, longLayout.fontSize) <= 260)).toBe(true);
  });

  it("keeps two lines at the minimum supported height", () => {
    const minimumHeightLayout = resolveOverlayDisplayText(
      "这是一个超过十八个字的实时字幕测试文本",
      36,
      400,
      measureByChar,
    );
    expect(minimumHeightLayout).toMatchObject({
      mode: "double",
      fontSize: 14,
      lineLimit: 2,
    });
    expect(minimumHeightLayout.lines).toHaveLength(2);
    expect(resolveOverlayDisplayText("第一行\n第二行", 72, 260, measureByChar)).toEqual({
      mode: "double",
      fontSize: 18,
      lineLimit: 2,
      lines: ["第一行", "第二行"],
    });
    expect(fitOverlayDisplayText("这是一个超过十八个字的实时字幕测试文本", 2, 18, 72, 400, measureByChar)).toEqual({
      fontSize: 18,
      lines: ["这是一个超过十八个字", "的实时字幕测试文本"],
    });
  });
});
