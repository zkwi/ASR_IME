# 悬浮字幕双行保障实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 保证长实时字幕在可显示双行时稳定显示两行，并避免异常短中间包清空已有上下文。

**Architecture:** Rust 窗口层统一计算不低于 `52px` 的有效高度，前端继续只根据真实 DOM 尺寸做无状态排版。实时字幕缓冲仅过滤单次会话内明显异常的 1-4 字骤降，不拼接文本且不影响最终包。

**Tech Stack:** Rust、Tauri 2、Svelte 5、TypeScript、Node.js 回归脚本。

## Global Constraints

- 不新增依赖和配置字段。
- 不恢复字幕滚动、历史锁定状态或多阶段排版状态。
- 不改变最终 ASR 文本、润色、粘贴和统计链路。
- 不记录或输出识别正文、密钥及其他隐私数据。

---

### Task 1: 最低有效字幕高度

**Files:**
- Modify: `src-tauri/src/config.rs`
- Modify: `src-tauri/src/config_validation.rs`
- Modify: `src-tauri/src/overlay.rs`
- Modify: `scripts/test-overlay-layout.mjs`

**Interfaces:**
- Produces: `config::MIN_UI_HEIGHT: u32` and `overlay::effective_overlay_height(u32) -> u32`。

- [x] **Step 1: 添加失败测试**

在 Rust 测试中断言 `51px` 配置不可保存、`52px` 可保存，并断言运行时把 `40px`、`51px` 钳制为 `52px`。前端布局脚本增加 `36px` 可用高度下长文本返回两行的断言。

- [x] **Step 2: 验证测试因缺少高度保障而失败**

Run: `cargo test overlay::tests::clamps_legacy_low_height_for_two_lines --manifest-path src-tauri/Cargo.toml`

Expected: FAIL，因为 `effective_overlay_height` 尚不存在。

Run: `cargo test rejects_overlay_height_that_cannot_show_two_lines --manifest-path src-tauri/Cargo.toml`

Expected: FAIL，因为当前仍接受 `51px`。

- [x] **Step 3: 实现最小高度保障**

在 `config.rs` 定义 `pub const MIN_UI_HEIGHT: u32 = 52;`；配置校验使用该下限；`overlay.rs` 在设置尺寸和计算纵向位置时使用 `ui.height.max(MIN_UI_HEIGHT)`。

- [x] **Step 4: 验证高度测试通过**

Run: `cargo test overlay::tests::clamps_legacy_low_height_for_two_lines --manifest-path src-tauri/Cargo.toml`

Expected: PASS。

Run: `cargo test rejects_overlay_height_that_cannot_show_two_lines --manifest-path src-tauri/Cargo.toml`

Expected: PASS。

Run: `npm run test:overlay`

Expected: `Overlay layout tests passed.`

### Task 2: 异常短中间包保护

**Files:**
- Modify: `src-tauri/src/asr_ws/partial_text.rs`

**Interfaces:**
- Consumes: 每次录音会话独立创建的 `LiveCaptionBuffer`。
- Produces: `LiveCaptionBuffer::update` 对明显异常的 1-4 字骤降返回 `None`。

- [x] **Step 1: 添加失败测试**

将“长字幕后接受非重合单字”的预期改为保留长字幕，并增加超过短片段阈值的正常修订仍可更新的测试。

- [x] **Step 2: 验证单字骤降测试失败**

Run: `cargo test live_caption_buffer_keeps_context_when_non_overlapping_tiny_fragment_arrives --manifest-path src-tauri/Cargo.toml`

Expected: FAIL，当前实现会返回单字更新。

- [x] **Step 3: 实现保守过滤**

当当前字幕不少于 8 字、下一条不超过 4 字且长度至少骤降 6 字时保留当前字幕；其他非重合更新继续正常通过。

- [x] **Step 4: 验证字幕缓冲测试通过**

Run: `cargo test live_caption_buffer --manifest-path src-tauri/Cargo.toml`

Expected: 所有字幕缓冲测试 PASS。

### Task 3: 文档、版本和发布

**Files:**
- Modify: `config.example.toml`
- Modify: `README.md`
- Modify: `README.en.md`
- Modify: `CHANGELOG.md`
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`
- Create: `docs/audits/2026-07-11-release-0.7.8-overlay-two-line-guard-audit.md`

**Interfaces:**
- Produces: 版本 `0.7.8` 和对应 Windows NSIS 安装包。

- [x] **Step 1: 同步用户文档和版本号**

说明字幕最低有效高度、旧配置兼容策略和异常短中间包保护；将五处版本号同步为 `0.7.8`。

- [x] **Step 2: 运行完整验证**

Run: `npm run ai:check`

Expected: PASS。

Run: `npm run ai:release-check`

Expected: PASS，Rust 审计仅允许项目已记录的上游警告。

Run: `npm run tauri -- build`

Expected: 生成 `src-tauri/target/release/bundle/nsis/VoxType_0.7.8_x64-setup.exe`。

- [x] **Step 3: 发布**

检查 `git status` 和 diff 后，使用简体中文提交信息；推送 `main`，创建并推送 `v0.7.8` 标签，创建 GitHub Release 并上传 NSIS 安装包。

- [x] **Step 4: 验证远端结果**

确认 Release 资产可见、校验和可读取，并等待对应 GitHub Actions 工作流完成。
