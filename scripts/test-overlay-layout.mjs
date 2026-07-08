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

  assert.equal(layout.preferredOverlayLineLimit(1, 72), 1);
  assert.equal(layout.preferredOverlayLineLimit(3, 72), 2);
  assert.equal(layout.preferredOverlayLineLimit(1, 72, true), 2);
  assert.equal(layout.preferredOverlayLineLimit(2, 33), 1);
  assert.equal(layout.preferredOverlayLineLimit(3, 20), 1);

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

  assert.deepEqual(layout.resolveOverlayLayout("短句", false, 72, 1), {
    mode: "single",
    fontSize: 20,
    lineLimit: 1,
  });

  assert.deepEqual(layout.resolveOverlayLayout("还有一个问题是，宝宝整体", false, 72, 1), {
    mode: "single",
    fontSize: 20,
    lineLimit: 1,
  });
  assert.deepEqual(
    layout.resolveOverlayLayout("这是一个超过十八个字的实时字幕测试文本", false, 72, 1),
    {
      mode: "double",
      fontSize: 20,
      lineLimit: 2,
    },
  );
  assert.deepEqual(
    layout.fitOverlayDisplayText("这是一个超过十八个字的实时字幕测试文本", 2, 20, 72, 400, measureByChar),
    { fontSize: 20, lines: ["这是一个超过十八个字", "的实时字幕测试文本"] },
  );
  assert.deepEqual(
    layout.fitOverlayDisplayText(
      layout.normalizeOverlayText("领导都这么说了，  要主动拥抱 。"),
      2,
      20,
      72,
      260,
      measureByChar,
    ),
    { fontSize: 18, lines: ["领导都这么说了，要主动拥抱。"] },
  );

  assert.deepEqual(layout.resolveOverlayLayout("一二三四五", false, 72, 3), {
    mode: "double",
    fontSize: 20,
    lineLimit: 2,
  });

  assert.deepEqual(layout.resolveOverlayLayout("一二三四五", false, 20, 3), {
    mode: "single",
    fontSize: 15,
    lineLimit: 1,
  });
} finally {
  await server.close();
}

console.log("Overlay layout tests passed.");
