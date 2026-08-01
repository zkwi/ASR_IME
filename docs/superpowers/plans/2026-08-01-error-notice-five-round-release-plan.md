# VoxType 0.10.4 错误通知最近五轮保留 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让主窗口底部错误通知只保留在从错误发生轮起算的最近五轮录音窗口，并发布 VoxType 0.10.4。

**Architecture:** 通知 controller 只保存当前通知及一个内存轮次余量；每次显示新错误时把余量重置为 5，每次真实的新录音会话开始时扣减，余量归零后复用现有 `clear()`。录音边沿由 `sessionState.ts` 的纯函数判定，`VoxTypeController.svelte.ts` 只负责把 phase 变化连接到通知 controller，避免修改 Rust 会话状态机。

**Tech Stack:** Svelte 5 runes、TypeScript、Vitest、Rust/Tauri 2、PowerShell、Git、GitHub CLI。

## Global Constraints

- 错误发生轮计入五轮窗口；错误在第 N 至 N+4 轮可见，第 N+5 轮开始时清除。
- 新错误替换旧错误并重置五轮；手动关闭立即清除。
- 成功、信息和警告继续使用现有定时策略；错误不增加固定秒数超时。
- 五轮状态只保存在当前窗口内存，不写配置、日志、统计或其他本地数据。
- 不新增依赖、配置字段或用户可见文案 key，不修改通知 DOM 与样式。
- 不修改 ASR 判错、最终包、LLM、剪贴板、统计、热键、托盘或 Rust 会话状态机。
- 版本从 `0.10.3` 升到 `0.10.4`，同步五处版本来源、CHANGELOG、双语 README、双语功能 Wiki、发布审计和文档索引。

---

### Task 1: 用失败测试固定错误通知的五轮窗口

**Files:**
- Create: `src/lib/utils/notificationController.test.ts`
- Modify: `src/lib/utils/notificationPolicy.test.ts`
- Modify: `src/lib/utils/notificationPolicy.ts`
- Modify: `src/lib/app/notificationController.svelte.ts`

**Interfaces:**
- Consumes: `createNotificationController(options)`、现有 `show()` 与 `clear()`。
- Produces: `ERROR_NOTICE_RETENTION_ROUNDS: 5`、`advanceSessionRound(): void`。

- [ ] **Step 1: 先把旧测试名称改成准确的时间策略描述**

将 `notificationPolicy.test.ts` 中第一项测试改为：

```ts
it("does not hide errors on a wall-clock timer", () => {
  expect(noticeAutoDismissMs("error", false, 12)).toBeNull();
});
```

这保留原有无固定秒数超时规则，同时不再错误声称错误只能由用户关闭。

- [ ] **Step 2: 创建真实 controller 的失败测试**

新增 `src/lib/utils/notificationController.test.ts`：

```ts
import { describe, expect, it } from "vitest";
import { createNotificationController } from "$lib/app/notificationController.svelte";

function createController() {
  return createNotificationController({
    t: () => "操作失败",
    setStatusMessage: () => undefined,
    logError: () => undefined,
  });
}

function advanceRounds(
  controller: ReturnType<typeof createNotificationController>,
  rounds: number,
) {
  for (let index = 0; index < rounds; index += 1) {
    controller.advanceSessionRound();
  }
}

describe("notification controller", () => {
  it("clears an error when it falls outside the five-round window", () => {
    const controller = createController();
    controller.show("偶发识别错误", "error");

    advanceRounds(controller, 4);
    expect(controller.message).toBe("偶发识别错误");

    controller.advanceSessionRound();
    expect(controller.message).toBe("");
  });

  it("restarts the five-round window when a newer error replaces the current one", () => {
    const controller = createController();
    controller.show("旧错误", "error");
    advanceRounds(controller, 4);

    controller.show("新错误", "error");
    advanceRounds(controller, 4);
    expect(controller.message).toBe("新错误");

    controller.advanceSessionRound();
    expect(controller.message).toBe("");
  });
});
```

- [ ] **Step 3: 运行红灯测试**

Run: `npm run test:unit -- src/lib/utils/notificationController.test.ts src/lib/utils/notificationPolicy.test.ts`

Expected: FAIL，`advanceSessionRound` 尚不存在；失败原因必须来自缺失行为，而不是测试环境或导入错误。

- [ ] **Step 4: 实现最小五轮状态**

在 `notificationPolicy.ts` 增加：

```ts
export const ERROR_NOTICE_RETENTION_ROUNDS = 5;
```

在 `notificationController.svelte.ts` 中导入该常量，增加：

```ts
let remainingErrorRounds = 0;
```

`show()` 设置当前通知后更新余量：

```ts
remainingErrorRounds = nextKind === "error" ? ERROR_NOTICE_RETENTION_ROUNDS : 0;
```

增加轮次推进方法：

```ts
function advanceSessionRound() {
  if (!message || kind !== "error") return;
  remainingErrorRounds = Math.max(0, remainingErrorRounds - 1);
  if (remainingErrorRounds === 0) clear();
}
```

`clear()` 同时执行 `remainingErrorRounds = 0`，并在返回对象中导出 `advanceSessionRound`。

- [ ] **Step 5: 运行绿灯测试**

Run: `npm run test:unit -- src/lib/utils/notificationController.test.ts src/lib/utils/notificationPolicy.test.ts`

Expected: 两个目标文件全部 PASS；错误在 4 次推进后仍存在，第 5 次推进后为空，新错误重新获得完整五轮。

---

### Task 2: 只在真实新录音边沿推进轮次

**Files:**
- Modify: `src/lib/utils/sessionState.test.ts`
- Modify: `src/lib/utils/sessionState.ts`
- Modify: `src/lib/app/VoxTypeController.svelte.ts`

**Interfaces:**
- Produces: `startsNewRecordingSession(previousPhase: SessionPhase, nextPhase: SessionPhase): boolean`。
- Consumes: Task 1 的 `notifications.advanceSessionRound()`。

- [ ] **Step 1: 添加新录音边沿失败测试**

在 `sessionState.test.ts` 导入 `startsNewRecordingSession`，增加：

```ts
it("counts only terminal-to-recording transitions as a new session", () => {
  expect(startsNewRecordingSession("idle", "starting")).toBe(true);
  expect(startsNewRecordingSession("failed", "recording")).toBe(true);
  expect(startsNewRecordingSession("succeeded", "starting")).toBe(true);

  expect(startsNewRecordingSession("starting", "recording")).toBe(false);
  expect(startsNewRecordingSession("recording", "starting")).toBe(false);
  expect(startsNewRecordingSession("stopping", "recording")).toBe(false);
  expect(startsNewRecordingSession("idle", "idle")).toBe(false);
});
```

- [ ] **Step 2: 运行红灯测试**

Run: `npm run test:unit -- src/lib/utils/sessionState.test.ts`

Expected: FAIL，`startsNewRecordingSession` 尚未导出。

- [ ] **Step 3: 实现纯边沿判断**

在 `sessionState.ts` 增加：

```ts
const sessionStartSourcePhases = new Set<SessionPhase>(["idle", "succeeded", "failed"]);
const sessionEntryPhases = new Set<SessionPhase>(["starting", "recording"]);

export function startsNewRecordingSession(previousPhase: SessionPhase, nextPhase: SessionPhase) {
  return sessionStartSourcePhases.has(previousPhase) && sessionEntryPhases.has(nextPhase);
}
```

- [ ] **Step 4: 连接主 controller**

在 `VoxTypeController.svelte.ts` 的 session-state imports 中加入 `startsNewRecordingSession`。把传给 `createSessionController()` 的 `setPhase` 改为：

```ts
setPhase: (value) => {
  if (startsNewRecordingSession(sessionPhase, value)) {
    notifications.advanceSessionRound();
  }
  sessionPhase = value;
},
```

这样 UI 点击、全局热键和 native event 最终都通过现有 `session.applyState()` 路径计数；`starting -> recording` 与重复事件不会二次扣减。

- [ ] **Step 5: 运行目标测试、类型检查和生产构建**

Run: `npm run test:unit -- src/lib/utils/notificationController.test.ts src/lib/utils/notificationPolicy.test.ts src/lib/utils/sessionState.test.ts`

Run: `npm run check`

Run: `npm run build`

Expected: 目标测试全部 PASS；Svelte/TypeScript 为 0 errors、0 warnings；Vite 生产构建成功。

- [ ] **Step 6: 检查范围并提交生产改动**

Run: `git diff --check`、`git diff --stat`、`git diff -- src/lib`

Stage only:

```powershell
git add -- src/lib/utils/notificationController.test.ts src/lib/utils/notificationPolicy.test.ts src/lib/utils/notificationPolicy.ts src/lib/app/notificationController.svelte.ts src/lib/utils/sessionState.test.ts src/lib/utils/sessionState.ts src/lib/app/VoxTypeController.svelte.ts
```

Commit: `修复错误通知最近五轮后自动清除`

---

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
- Create: `docs/audits/2026-08-01-release-0.10.4-error-notice-retention-audit.md`

**Interfaces:**
- Produces: 一致的 `0.10.4` 版本元数据、双语行为说明、CHANGELOG 和发布审计入口。

- [ ] **Step 1: 同步双语用户行为说明**

中文 README 和功能 Wiki 使用：

```text
主窗口底部只显示最新一条错误通知。错误发生轮计入最近 5 轮窗口，新错误会重新开始计数；到第 6 轮开始时自动清除，也可随时手动关闭。
```

英文 README 和功能 Wiki 使用：

```text
The main window shows only the latest error notice. Its originating recording counts toward a five-session window, a newer error restarts the count, and the notice clears when the sixth session starts or when closed manually.
```

- [ ] **Step 2: 同步版本来源和 CHANGELOG**

Run: `npm version 0.10.4 --no-git-tag-version`

仅将 `src-tauri/Cargo.toml`、根包对应的 `src-tauri/Cargo.lock` 和 `src-tauri/tauri.conf.json` 从 `0.10.3` 改为 `0.10.4`；不得改动依赖包 `@tybys/wasm-util` 自身的 `0.10.3`。

在 CHANGELOG 的 `[未发布]` 后增加：

```markdown
## [0.10.4] - 2026-08-01

### 修复

- 主窗口底部错误通知不再永久驻留：错误发生轮计入最近 5 轮窗口，新错误重新计数，第 6 轮开始时自动清除，手动关闭仍立即生效。

### 工程治理

- 新增错误通知五轮边界、新错误重置和录音状态边沿回归测试；通知计数只保存在当前窗口内存，不写入配置、日志或统计。
```

- [ ] **Step 3: 创建并索引发布审计**

创建 `docs/audits/2026-08-01-release-0.10.4-error-notice-retention-audit.md`，记录：patch 版本理由、永久驻留根因、五轮边界、改动文件、主链路/配置/隐私边界、TDD 红绿证据、UI 验证、发布门禁、安装包大小与 SHA-256、未签名状态和手工回归建议。

在 `docs/README.md` 的发布审计列表顶部加入 0.10.4 链接。

- [ ] **Step 4: 运行日常完整检查**

Run: `npm run ai:check`

Expected: 前端检查、生产构建、Vitest、治理检查、密钥扫描、Rust fmt/check/test 全部成功。

---

### Task 4: 真实页面验证、发布门禁、安装包与 GitHub Release

**Files:**
- Verify: Tasks 1-3 的全部文件。
- Build artifact: `src-tauri/target/release/bundle/nsis/VoxType_0.10.4_x64-setup.exe`

**Interfaces:**
- Produces: 发布提交、远端 `main`、annotated tag `v0.10.4`、正式 GitHub Release 和 NSIS 安装包资产。

- [ ] **Step 1: 运行真实浏览器烟雾验证**

使用项目现有 Vite dev server 和 Playwright 打开主页面，检查首页正常加载、控制台无 error、底部通知样式没有布局回退。生命周期行为由真实 notification controller 的 Vitest 和录音边沿纯函数测试验证；不调用云端 ASR/LLM，不读取凭据或麦克风。

- [ ] **Step 2: 运行完整发布门禁**

Run: `npm run ai:release-check`

Expected: 发布前置诊断、日常检查、npm/Rust 审计、Clippy (`-D warnings`) 和 Tauri debug build 全部成功；Rust audit 只允许仓库清单已有的上游维护状态 warning。

- [ ] **Step 3: 构建正式安装包并核验元数据**

Run: `npm run tauri -- build`

Run:

```powershell
$installerPath = Resolve-Path 'src-tauri/target/release/bundle/nsis/VoxType_0.10.4_x64-setup.exe'
Get-Item -LiteralPath $installerPath | Select-Object FullName, Length, LastWriteTime
Get-FileHash -LiteralPath $installerPath -Algorithm SHA256
Get-AuthenticodeSignature -LiteralPath $installerPath | Select-Object Status, StatusMessage
```

Expected: 安装包存在且大小大于 0；SHA-256 为 64 位十六进制；签名状态按当前个人项目发布方式记录。

- [ ] **Step 4: 写入实际发布证据并重新验证最终树**

把实际测试数量、门禁结果、安装包大小、SHA-256、签名状态和浏览器结果写入 0.10.4 发布审计。

Run: `npm run ai:check`

Run: `git diff --check`、`git status --short`、`git diff --stat`、`git diff`

Expected: 最终内容全部通过；变更只包含本计划列出的源码、测试、文档和版本文件，不包含密钥、识别正文或无关文件。

- [ ] **Step 5: 提交并推送发布内容**

显式 stage 本计划剩余文档、版本和审计文件，提交：`发布 0.10.4 优化错误通知生命周期`。

Run: `git push origin main`

等待该 commit 的 GitHub Actions `Release-grade check suite` 成功后再创建标签。

- [ ] **Step 6: 创建标签和正式 Release**

Run:

```powershell
git tag -a v0.10.4 -m "发布 VoxType 0.10.4"
git push origin v0.10.4
```

使用 `gh release create v0.10.4` 创建名为 `VoxType 0.10.4` 的非草稿、非预发布 Release，上传 `VoxType_0.10.4_x64-setup.exe`。Release notes 概括五轮自动清理、最新错误替换、无配置与主链路改动，以及发布验证结果。

- [ ] **Step 7: 验证远端发布**

Run: `gh release view v0.10.4 --json name,tagName,isDraft,isPrerelease,assets,url,publishedAt,targetCommitish`

Run: `git ls-remote origin refs/heads/main refs/tags/v0.10.4 refs/tags/v0.10.4^{}`

Expected: Release 非草稿、非预发布，标签和远端 main 指向本地发布提交，安装包资产名称及大小与本地一致；工作树干净。
