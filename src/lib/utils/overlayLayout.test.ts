import { describe, expect, it } from "vitest";
import {
  balanceOverlayLines,
  fitOverlayDisplayText,
  normalizeOverlayText,
  resolveOverlayDisplayText,
  wrapOverlayText,
} from "./overlayLayout";

const measureByChar = (text: string, fontSize: number) => Array.from(text).length * fontSize;
const measureMixedWidth = (text: string, fontSize: number) =>
  Array.from(text).reduce(
    (width, char) => width + (char.codePointAt(0)! <= 0x7f ? fontSize / 2 : fontSize),
    0,
  );

describe("overlay layout", () => {
  it("collapses preview line breaks and inline spacing", () => {
    expect(normalizeOverlayText("第一行\r\n第二行")).toBe("第一行第二行");
    expect(normalizeOverlayText("第一行\n\n\t第二行")).toBe("第一行第二行");
    expect(normalizeOverlayText("hello\rworld")).toBe("hello world");
    expect(normalizeOverlayText("领导都这么说了，  要主动拥抱 。")).toBe("领导都这么说了，要主动拥抱。");
  });

  it("distributes two lines by measured width without leaving a tiny tail", () => {
    expect(balanceOverlayLines("这是一个超过十八个字的实时字幕测试文本", 18, 300, measureByChar)).toEqual([
      "这是一个超过十八个字的实时字幕测",
      "试文本",
    ]);
    expect(balanceOverlayLines("还有一个问题是，宝宝整体", 18, 180, measureByChar)).toEqual([
      "还有一个问题是，宝",
      "宝整体",
    ]);
    // 标点不能出现在行首
    expect(balanceOverlayLines("你好，世界", 18, 60, measureByChar)).toEqual(["你好，", "世界"]);
    // 按真实测量宽度而不是字符数平分
    expect(balanceOverlayLines("中文ABCDEFGH", 18, 90, measureMixedWidth)).toEqual([
      "中文ABCDE",
      "FGH",
    ]);
    expect(balanceOverlayLines("字", 18, 400, measureByChar)).toEqual(["字"]);
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
    // 12 字在 460px 宽悬浮窗的 372px 文字区只占 240px（64%），仍然保持单行
    expect(resolveOverlayDisplayText("还有一个问题是，宝宝整体", 72, 372, measureByChar)).toEqual({
      mode: "single",
      fontSize: 20,
      lineLimit: 1,
      lines: ["还有一个问题是，宝宝整体"],
    });
  });

  it("keeps measured text on one line until it actually overflows", () => {
    const eighteenChars = "甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳午未";
    const nineteenChars = `${eighteenChars}申`;

    expect(resolveOverlayDisplayText(eighteenChars, 72, 372, measureByChar).mode).toBe("single");
    expect(resolveOverlayDisplayText(nineteenChars, 72, 400, measureByChar).mode).toBe("single");
    expect(resolveOverlayDisplayText(eighteenChars, 72, 341, measureByChar).mode).toBe("double");
  });

  it("uses the actual default overlay geometry when selecting line count", () => {
    // 仓库默认悬浮窗为 350px，扣除内边距、音量计、间距和文字右内边距后内容区为 262px
    expect(resolveOverlayDisplayText("甲乙丙丁戊己庚辛壬癸子丑寅", 48, 262, measureByChar).mode).toBe("single");
    expect(resolveOverlayDisplayText("甲乙丙丁戊己庚辛壬癸子丑寅卯", 48, 262, measureByChar).mode).toBe("double");
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
      300,
      measureByChar,
    );
    expect(doubleLayout.mode).toBe("double");
    expect(doubleLayout.lineLimit).toBe(2);
    expect(doubleLayout.fontSize).toBeLessThanOrEqual(shortLayout.fontSize);
    expect(doubleLayout.lines.join("")).toBe("这是一个超过十八个字的实时字幕测试文本");
    expect(measureByChar(doubleLayout.lines[0], doubleLayout.fontSize) / 300).toBeGreaterThanOrEqual(0.9);
    expect(Array.from(doubleLayout.lines[1]).length).toBeGreaterThanOrEqual(3);

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

  it("uses a nearly full 19px single line before wrapping", () => {
    const nineteenChars = "甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳午未申";
    const compact = resolveOverlayDisplayText(nineteenChars, 72, 372, measureByChar);
    expect(compact).toEqual({
      mode: "single",
      fontSize: 19,
      lineLimit: 1,
      lines: [nineteenChars],
    });
    expect(measureByChar(compact.lines[0], compact.fontSize) / 372).toBeGreaterThan(0.95);

    const twentyChars = `${nineteenChars}酉`;
    const wrapped = resolveOverlayDisplayText(twentyChars, 72, 372, measureByChar);
    expect(wrapped).toMatchObject({ mode: "double", fontSize: 20, lineLimit: 2 });
    expect(measureByChar(wrapped.lines[0], wrapped.fontSize) / 372).toBeGreaterThanOrEqual(0.9);
    expect(Array.from(wrapped.lines[1]).length).toBeGreaterThanOrEqual(3);
    expect(wrapped.lines.join("")).toBe(twentyChars);
  });

  it("uses two lines instead of shrinking a full single line below 19px", () => {
    // 16 字在 300px 宽度下放不进 20px 或 19px 单行；不得继续降到 18px 硬撑
    const layout = resolveOverlayDisplayText("这是一个需要换行显示的实时字幕文", 72, 300, measureByChar);
    expect(layout.mode).toBe("double");
    expect(layout.lines).toHaveLength(2);
  });

  it("uses the first line without leaving an orphan tail", () => {
    // 21 字在 372px/18px 下贪心换行会得到 20 + 1，视觉上仍像一行
    const layout = resolveOverlayDisplayText(
      "是要保证两行显示，而不是只显示一行。那现在",
      72,
      372,
      measureByChar,
    );
    expect(layout.lines).toHaveLength(2);
    const first = measureByChar(layout.lines[0], layout.fontSize);
    expect(first / 372).toBeGreaterThanOrEqual(0.9);
    expect(Array.from(layout.lines[1]).length).toBeGreaterThanOrEqual(3);
    expect(layout.lines.join("")).toBe("是要保证两行显示，而不是只显示一行。那现在");
  });

  it("uses the first line naturally when text only slightly exceeds one line", () => {
    // 截图对应约 402×76 的窗口，文字区为 314×60；整句放不进一行，但对半拆会让两行都只占约一半宽度
    const layout = resolveOverlayDisplayText(
      "请仿照这个 UI 对整体 UI 风格进行调整。",
      60,
      314,
      measureMixedWidth,
    );
    expect(layout).toMatchObject({ mode: "double", fontSize: 20, lineLimit: 2 });
    expect(layout.lines).toHaveLength(2);
    expect(layout.lines.join("")).toBe("请仿照这个 UI 对整体 UI 风格进行调整。");
    expect(measureMixedWidth(layout.lines[0], layout.fontSize) / 314).toBeGreaterThanOrEqual(0.9);
    expect(Array.from(layout.lines[1].replace(/\s/g, "")).length).toBeGreaterThanOrEqual(3);
  });

  it("shrinks slightly instead of putting punctuation at the start of line two", () => {
    // 18px 下只有 5+5 能放入两行，唯一切点恰好在逗号前；缩到 15px 后可把逗号留在第一行
    const layout = resolveOverlayDisplayText("一二三四五，六七八九", 72, 90, measureByChar);
    expect(layout).toEqual({
      mode: "double",
      fontSize: 15,
      lineLimit: 2,
      lines: ["一二三四五，", "六七八九"],
    });
  });

  it("fills both lines with the newest text when it overflows", () => {
    // 30 字在 260px/18px 下需要三行，贪心取尾会只剩「满行 + 2 字」，上下文白白少了半行
    const layout = resolveOverlayDisplayText(
      "这是一段明显超过两行容量的很长的实时字幕文本内容需要滚动显示尾部",
      72,
      260,
      measureByChar,
    );
    expect(layout.lines).toHaveLength(2);
    const [first, second] = layout.lines.map((line) => Array.from(line).length);
    expect(Math.abs(first - second)).toBeLessThanOrEqual(1);
    // 显示的是最新的尾部，且尽量填满两行
    expect(layout.lines.join("")).toBe("明显超过两行容量的很长的实时字幕文本内容需要滚动显示尾部");
  });

  it("bounds measured text work for long live captions", () => {
    const text = "这是一段用于悬浮字幕布局性能审计的实时识别文本".repeat(50);
    let measuredChars = 0;
    const countingMeasure = (value: string, fontSize: number) => {
      const length = Array.from(value).length;
      measuredChars += length;
      return length * fontSize;
    };

    const layout = resolveOverlayDisplayText(text, 60, 314, countingMeasure);

    expect(layout).toMatchObject({ mode: "double", lineLimit: 2 });
    expect(layout.lines).toHaveLength(2);
    expect(measuredChars).toBeLessThan(Array.from(text).length * 50);
  });

  it("keeps a punctuation-heavy tail within the measured two-line width", () => {
    // 连续标点会让贪心换行数低估真实宽度，旧尾部二分会退化成一条超宽单行
    const layout = resolveOverlayDisplayText("一二三四五！！！！！！六七八九十", 72, 90, measureByChar);
    expect(layout.lines).toHaveLength(2);
    expect(layout.lines.every((line) => measureByChar(line, layout.fontSize) <= 90)).toBe(true);
    expect(layout.lines.join("")).toMatch(/六七八九十$/);
  });

  it("keeps two lines at the minimum supported height", () => {
    const minimumHeightLayout = resolveOverlayDisplayText(
      "这是一个超过十八个字的实时字幕测试文本",
      36,
      300,
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
      fontSize: 20,
      lineLimit: 2,
      lines: ["第一行", "第二行"],
    });
    expect(fitOverlayDisplayText("这是一个超过十八个字的实时字幕测试文本", 2, 18, 72, 400, measureByChar)).toEqual({
      fontSize: 18,
      lines: ["这是一个超过十八个字的实时字幕测", "试文本"],
    });
  });
});
