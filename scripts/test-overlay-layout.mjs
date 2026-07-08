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
  assert.deepEqual(
    layout.splitOverlayLine("还有一个问题是，宝宝整体"),
    ["还有一个问题是，", "宝宝整体"],
  );
  assert.deepEqual(layout.splitOverlayLine("你好，世界"), ["你好，世界"]);
  assert.deepEqual(
    layout.rebalanceOverlayDisplayLines(["还有一个问题是，宝宝整体"], 2),
    ["还有一个问题是，", "宝宝整体"],
  );
  assert.deepEqual(
    layout.rebalanceOverlayDisplayLines(["显反转效应的东西动量也会有"], 2, true),
    ["显反转效应的", "东西动量也会有"],
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
    mode: "double",
    fontSize: 20,
    lineLimit: 2,
  });
  assert.deepEqual(layout.resolveOverlayLayout("还有一个问题是，宝宝整体", false, 48, 1), {
    mode: "double",
    fontSize: 19,
    lineLimit: 2,
  });
  const screenshotTextLayout = layout.resolveOverlayLayout("还有一个问题是，宝宝整体", false, 72, 1);
  const screenshotTextLines = layout.rebalanceOverlayDisplayLines(
    layout.wrapOverlayText("还有一个问题是，宝宝整体", screenshotTextLayout.fontSize, 400, measureByChar),
    screenshotTextLayout.lineLimit,
  );
  assert.deepEqual(screenshotTextLines, ["还有一个问题是，", "宝宝整体"]);
  assert.deepEqual(
    layout.fitOverlayDisplayText("还有一个问题是，宝宝整体", 2, 20, 72, 400, measureByChar),
    { fontSize: 20, lines: ["还有一个问题是，", "宝宝整体"] },
  );
  assert.deepEqual(
    layout.fitOverlayDisplayText("还有一个问题是，宝宝整体", 2, 20, 48, 400, measureByChar),
    { fontSize: 19, lines: ["还有一个问题是，", "宝宝整体"] },
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
