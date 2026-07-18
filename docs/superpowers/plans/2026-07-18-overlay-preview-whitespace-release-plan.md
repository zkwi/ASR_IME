# VoxType 0.8.2 悬浮字幕预览空白压缩 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在悬浮字幕预览阶段压缩 ASR 返回的格式性换行和连续空白，让短文本稳定显示一行、长文本稳定显示两行，并发布 VoxType 0.8.2。

**Architecture:** 只在前端 `normalizeOverlayText` 边界归一化预览字符串，后续继续复用现有长度、宽度、高度和两行上限算法。原始 ASR 最终文本及润色、粘贴、最近上下文、自动热词、统计和日志链路保持不变。

**Tech Stack:** Svelte 5、TypeScript、Vitest、Rust/Tauri 2、PowerShell、GitHub CLI。

## Global Constraints

- 不新增依赖、配置字段或用户可见文案 key。
- 不修改 ASR 分片、最终包门禁、最终文本选择、LLM、剪贴板、热键、托盘、统计或日志。
- 不修改悬浮字幕 DOM、CSS、窗口高度和字号阈值。
- 识别正文仅用于内存中的预览计算，不写入日志、诊断报告、统计或发布审计。
- 版本从 `0.8.1` 升为 `0.8.2`，同步五处版本来源、CHANGELOG、发布审计和文档索引。

---

### Task 1: 用失败测试固定预览空白压缩规则

**Files:**
- Modify: `src/lib/utils/overlayLayout.test.ts`
- Test: `src/lib/utils/overlayLayout.test.ts`

**Interfaces:**
- Consumes: `normalizeOverlayText(text: string): string`、`resolveOverlayDisplayText(...)`。
- Produces: 换行压缩、短文本单行、长文本双行且无空白显示行的回归约束。

- [x] **Step 1: 修改归一化测试**

将现有“保留换行”预期改为预览压缩，并覆盖中文、英文和连续空白：

```ts
it("collapses preview line breaks and inline spacing", () => {
  expect(normalizeOverlayText("第一行\r\n第二行")).toBe("第一行第二行");
  expect(normalizeOverlayText("第一行\n\n\t第二行")).toBe("第一行第二行");
  expect(normalizeOverlayText("hello\rworld")).toBe("hello world");
  expect(normalizeOverlayText("领导都这么说了，  要主动拥抱 。")).toBe("领导都这么说了，要主动拥抱。");
});
```

- [x] **Step 2: 增加布局行为测试**

```ts
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
```

- [x] **Step 3: 运行红灯测试**

Run: `npm run test:unit -- src/lib/utils/overlayLayout.test.ts`

Expected: FAIL；当前实现返回带 `\n` 的文本，并把短换行文本判为双行。

### Task 2: 最小实现预览空白压缩

**Files:**
- Modify: `src/lib/utils/overlayLayout.ts:18-34`
- Test: `src/lib/utils/overlayLayout.test.ts`

**Interfaces:**
- Produces: `normalizeOverlayText(text: string): string` 返回无换行、连续空白已折叠的预览字符串。

- [x] **Step 1: 替换预览归一化实现**

```ts
export function normalizeOverlayText(text: string) {
  const collapsed = String(text || "").replace(/\s+/g, " ").trim();
  return collapsed ? normalizeOverlayInlineSpacing(collapsed) : "";
}
```

- [x] **Step 2: 运行绿灯测试**

Run: `npm run test:unit -- src/lib/utils/overlayLayout.test.ts`

Expected: 目标文件全部测试 PASS，短文本一行、长文本两行且不存在空白显示行。

- [x] **Step 3: 运行前端类型与构建检查**

Run: `npm run check`

Expected: Svelte/TypeScript 检查 0 errors、0 warnings。

Run: `npm run build`

Expected: Vite 生产构建成功。

### Task 3: 同步用户文档、版本和发布记录

**Files:**
- Modify: `README.md`
- Modify: `README.en.md`
- Modify: `docs/wiki/Feature-Guide.md`
- Modify: `docs/wiki/Feature-Guide-English.md`
- Modify: `CHANGELOG.md`
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `docs/README.md`
- Create: `docs/audits/2026-07-18-release-0.8.2-overlay-preview-whitespace-audit.md`

**Interfaces:**
- Produces: 一致的 `0.8.2` 版本元数据、双语用户说明、CHANGELOG 和可核验发布审计。

- [x] **Step 1: 更新双语功能说明**

中文统一描述为：

```text
字幕会先压缩 ASR 中间态中的格式性换行和连续空白，再清理中文间多余空格；短文本保持单行，长文本或按实际宽度换行的文本最多显示两行。
```

英文统一描述为：

```text
Captions collapse formatting line breaks and repeated whitespace from interim ASR text, keep short text on one line, and use up to two visible lines for long or width-wrapped text.
```

- [x] **Step 2: 同步版本与更新日志**

将 `package.json`、`package-lock.json` 根包两处、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock` 根包和 `src-tauri/tauri.conf.json` 从 `0.8.1` 更新为 `0.8.2`。在 CHANGELOG 增加 `2026-07-18` 修复记录，说明换行和空白只在预览层压缩，不改变最终识别文本。

- [x] **Step 3: 创建并索引发布审计**

发布审计记录版本选择、根因、改动范围、隐私边界、TDD 红绿证据、完整门禁结果、安装包大小与 SHA-256；`docs/README.md` 增加 0.8.2 索引。

### Task 4: 完整验证、构建和 GitHub Release

**Files:**
- Verify: all files changed by Tasks 1-3
- Build artifact: `src-tauri/target/release/bundle/nsis/VoxType_0.8.2_x64-setup.exe`

**Interfaces:**
- Produces: Git commit、远端 `main`、`v0.8.2` 标签、GitHub Release 和 NSIS 安装包资产。

- [x] **Step 1: 运行完整日常检查**

Run: `npm run ai:check`

Expected: 全部本地阶段成功，包含前端检查、构建、Vitest、治理、密钥扫描和 Rust fmt/check/test。

- [x] **Step 2: 运行发布门禁**

Run: `npm run ai:release-check`

Expected: 发布前置诊断、日常检查、npm/Rust 审计、Clippy 和 Tauri debug 构建全部成功；仅允许仓库已记录的上游 Rust audit warning。

- [x] **Step 3: 构建正式安装包并记录校验值**

Run: `npm run tauri -- build`

Expected: 生成 `src-tauri/target/release/bundle/nsis/VoxType_0.8.2_x64-setup.exe`。

Run: `$installer = Resolve-Path 'src-tauri/target/release/bundle/nsis/VoxType_0.8.2_x64-setup.exe'; Get-Item -LiteralPath $installer; Get-FileHash -LiteralPath $installer -Algorithm SHA256`

Expected: 文件存在、大小大于 0、SHA-256 为 64 位十六进制字符串；把实际值写入发布审计。

- [x] **Step 4: 在审计定稿后重新验证**

Run: `npm run ai:check`

Expected: 最终工作树内容全部通过。

- [ ] **Step 5: 检查范围、提交并推送**

Run: `git status --short`、`git diff --check`、`git diff --stat`、`git diff`

Expected: 只包含本计划列出的源码、测试、文档和版本文件，无密钥、正文或无关改动。

Commit message: `发布 0.8.2 修复悬浮字幕换行预览`

Push: `git push origin main`

- [ ] **Step 6: 创建标签和 Release**

创建并推送 annotated tag `v0.8.2`，使用 GitHub CLI 创建名为 `VoxType 0.8.2` 的正式 Release，上传 NSIS 安装包，并在说明中概括换行压缩、单/双行规则和主链路不变。

- [ ] **Step 7: 验证远端发布**

Run: `gh release view v0.8.2 --json name,tagName,isDraft,isPrerelease,assets,url,publishedAt`

Expected: 非草稿、非预发布、标签为 `v0.8.2`，安装包资产存在且大小与本地一致；远端 `main`、标签提交与本地 HEAD 一致。
