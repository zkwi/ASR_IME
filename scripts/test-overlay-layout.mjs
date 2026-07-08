import assert from "node:assert/strict";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const server = await createServer({
  appType: "custom",
  configFile: false,
  resolve: {
    alias: {
      $lib: resolve(repoRoot, "src/lib"),
    },
  },
  server: { hmr: false, middlewareMode: true },
  logLevel: "error",
});

try {
  const layout = await server.ssrLoadModule("/src/lib/utils/overlayLayout.ts");
  const measureByChar = (text, fontSize) => Array.from(text).length * fontSize;

  assert.equal(layout.normalizeOverlayText("第一行\r\n第二行"), "第一行\n第二行");
  assert.equal(
    layout.normalizeOverlayText("领导都这么说了，  要主动拥抱 。"),
    "领导都这么说了，要主动拥抱。",
  );
  assert.deepEqual(
    layout.splitOverlayLine("还有一个问题是，宝宝整体"),
    ["还有一个问题是，宝宝整体"],
  );
  assert.deepEqual(layout.splitOverlayLine("你好，世界"), ["你好，世界"]);
  assert.deepEqual(
    layout.splitOverlayLine("这是一个超过十八个字的实时字幕测试文本"),
    ["这是一个超过十八个字", "的实时字幕测试文本"],
  );
  assert.deepEqual(
    layout.rebalanceOverlayDisplayLines(["还有一个问题是，宝宝整体"], 2),
    ["还有一个问题是，宝宝整体"],
  );
  assert.deepEqual(
    layout.rebalanceOverlayDisplayLines(["这是一个超过十八个字的实时字幕测试文本"], 2),
    ["这是一个超过十八个字", "的实时字幕测试文本"],
  );

  assert.deepEqual(
    layout.wrapOverlayText("一二三四五", 20, 40, measureByChar),
    ["一二", "三四", "五"],
  );

  assert.deepEqual(
    layout.wrapOverlayText("第一行\n第二行", 20, 200, measureByChar),
    ["第一行", "第二行"],
  );

  const shortLayout = layout.resolveOverlayDisplayText("短句", 72, 260, measureByChar);
  assert.deepEqual(shortLayout, {
    mode: "single",
    fontSize: 20,
    lineLimit: 1,
    lines: ["短句"],
  });

  assert.deepEqual(layout.resolveOverlayDisplayText("还有一个问题是，宝宝整体", 72, 260, measureByChar), {
    mode: "single",
    fontSize: 20,
    lineLimit: 1,
    lines: ["还有一个问题是，宝宝整体"],
  });

  const doubleLayout = layout.resolveOverlayDisplayText(
    "这是一个超过十八个字的实时字幕测试文本",
    72,
    400,
    measureByChar,
  );
  assert.equal(doubleLayout.mode, "double");
  assert.equal(doubleLayout.lineLimit, 2);
  assert.ok(doubleLayout.fontSize <= shortLayout.fontSize);
  assert.deepEqual(doubleLayout.lines, ["这是一个超过十八个字", "的实时字幕测试文本"]);

  assert.deepEqual(
    layout.resolveOverlayDisplayText("第一行\n第二行", 72, 260, measureByChar),
    { mode: "double", fontSize: 18, lineLimit: 2, lines: ["第一行", "第二行"] },
  );

  assert.deepEqual(
    layout.fitOverlayDisplayText("这是一个超过十八个字的实时字幕测试文本", 2, 18, 72, 400, measureByChar),
    { fontSize: 18, lines: ["这是一个超过十八个字", "的实时字幕测试文本"] },
  );
  const longLayout = layout.resolveOverlayDisplayText(
    "这是一段更长的实时字幕测试文本，用来模拟较长口述时的两行显示效果",
    72,
    260,
    measureByChar,
  );
  assert.equal(longLayout.mode, "double");
  assert.equal(longLayout.fontSize, 18);
  assert.equal(longLayout.lineLimit, 2);
  assert.equal(longLayout.lines.length, 2);
  assert.ok(longLayout.lines.every((line) => measureByChar(line, longLayout.fontSize) <= 260));
} finally {
  await server.close();
}

console.log("Overlay layout tests passed.");
