# VoxType 0.8.3 FunASR 累计字幕 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 FunASR 实时字幕持续显示已确认分句与当前识别分句的累计文本，并发布 VoxType 0.8.3。

**Architecture:** 在 `AliyunFinalGate` 生成仅供 UI 使用的累计文本，复用其已有 `final_sentences`，不在通用 `LiveCaptionBuffer` 中猜测或拼接 provider 片段。最终文本仍只在 `task-finished` 后由已确认分句生成，豆包、前端布局和输出主链路保持不变。

**Tech Stack:** Rust、Tauri 2、TypeScript/Vitest、PowerShell、GitHub CLI。

## Global Constraints

- 不调整前端 `SINGLE_LINE_MAX_CHARS = 18`、DOM、CSS、窗口尺寸或字号算法。
- 不修改 FunASR WebSocket 请求、音频节奏、`sentence_end` 判定、`task-finished` 门禁或最终文本内容。
- 不修改豆包 ASR、LLM、剪贴板、最近上下文、自动热词、统计、热键、托盘或日志。
- 不新增依赖、配置字段、用户可见文案 key 或配置迁移。
- 识别正文不得进入日志、诊断报告、统计、文档和发布资产；测试仅使用合成文本。
- 版本从 `0.8.2` 升为 `0.8.3`，同步五处版本来源、CHANGELOG、发布审计和文档索引。

---

### Task 1: 用失败测试复现 FunASR 跨分句字幕丢失

**Files:**
- Modify: `src-tauri/src/aliyun_asr.rs`
- Test: `src-tauri/src/aliyun_asr.rs`

**Interfaces:**
- Consumes: `AliyunFinalGate::apply(AliyunServerEvent)` 返回可选的 `ProviderSurfaceEvent`。
- Produces: 已确认第一句后，第二句 partial/stable 的显示事件必须携带累计文本的回归约束。

- [x] **Step 1: 添加跨分句累计显示测试**

```rust
#[test]
fn live_caption_accumulates_confirmed_sentences_with_current_partial() {
    let mut gate = AliyunFinalGate::default();

    assert_eq!(
        gate.apply(AliyunServerEvent::ResultGenerated {
            text: "第一句已经确认。".to_string(),
            sentence_end: true,
        }),
        Some(ProviderSurfaceEvent::StableText(
            "第一句已经确认。".to_string()
        ))
    );
    assert_eq!(
        gate.apply(AliyunServerEvent::ResultGenerated {
            text: "第二句正在识别".to_string(),
            sentence_end: false,
        }),
        Some(ProviderSurfaceEvent::PartialText(
            "第一句已经确认。第二句正在识别".to_string()
        ))
    );
    assert_eq!(gate.final_text().unwrap(), None);
    assert_eq!(
        gate.apply(AliyunServerEvent::ResultGenerated {
            text: "第二句已经完成。".to_string(),
            sentence_end: true,
        }),
        Some(ProviderSurfaceEvent::StableText(
            "第一句已经确认。第二句已经完成。".to_string()
        ))
    );
    assert_eq!(gate.final_text().unwrap(), None);
}
```

- [x] **Step 2: 运行红灯测试**

Run: `cargo test live_caption_accumulates_confirmed_sentences_with_current_partial --manifest-path src-tauri/Cargo.toml`

Expected: FAIL；当前第二句 partial 只返回 `第二句正在识别`，第二句 stable 只返回 `第二句已经完成。`。

### Task 2: 最小实现 FunASR 累计显示文本

**Files:**
- Modify: `src-tauri/src/aliyun_asr.rs:73-123`
- Test: `src-tauri/src/aliyun_asr.rs`

**Interfaces:**
- Produces: `AliyunFinalGate::accumulated_text` 接收可选字符串引用并返回 `String`，仅供显示事件和最终已确认文本复用。

- [x] **Step 1: 增加累计文本辅助方法并改写显示事件**

```rust
fn accumulated_text(&self, current: optional_string_reference) -> String {
    let mut text = self.final_sentences.concat();
    if let Some(current) = current {
        text.push_str(current);
    }
    text.trim().to_string()
}
```

`sentence_end=true` 时先将当前句加入 `final_sentences`，再返回 `StableText(self.accumulated_text(None))`；未结束句返回 `PartialText(self.accumulated_text(Some(&text)))`。`final_text()` 继续检查 `task_finished`，通过后返回 `self.accumulated_text(None)`。

- [x] **Step 2: 运行聚焦绿灯测试**

Run: `cargo test live_caption_accumulates_confirmed_sentences_with_current_partial --manifest-path src-tauri/Cargo.toml`

Expected: PASS。

Run: `cargo test aliyun_asr::tests --manifest-path src-tauri/Cargo.toml`

Expected: 全部 FunASR 单元测试 PASS，最终文本门禁用例保持通过。

### Task 3: 同步双语文档、版本和发布审计

**Files:**
- Modify: `README.md`
- Modify: `README.en.md`
- Modify: `docs/wiki/Setup-Guide.md`
- Modify: `docs/wiki/Setup-Guide-English.md`
- Modify: `CHANGELOG.md`
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `docs/README.md`
- Create: `docs/audits/2026-07-18-release-0.8.3-funasr-cumulative-caption-audit.md`

**Interfaces:**
- Produces: 一致的 `0.8.3` 版本元数据、双语 FunASR 行为说明、CHANGELOG 和发布审计。

- [x] **Step 1: 更新双语用户说明**

中文说明加入：

```text
FunASR 实时字幕会把已确认分句与当前未完成分句合并显示，但最终粘贴仍等待 task-finished。
```

英文说明加入：

```text
FunASR live captions combine confirmed sentences with the current unfinished sentence, while final paste still waits for task-finished.
```

- [x] **Step 2: 同步版本和 CHANGELOG**

将 `package.json`、`package-lock.json` 根包两处、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock` 根包和 `src-tauri/tauri.conf.json` 从 `0.8.2` 更新为 `0.8.3`。CHANGELOG 记录 FunASR 字幕累计修复和最终门禁不变。

- [x] **Step 3: 创建并索引发布审计**

审计记录根因、TDD 红绿证据、主链路与隐私边界、门禁结果、未执行真实 FunASR 麦克风测试的原因、安装包大小和 SHA-256；`docs/README.md` 增加 0.8.3 索引。

### Task 4: 完整验证、构建和发布 0.8.3

**Files:**
- Verify: all files changed by Tasks 1-3
- Build artifact: `src-tauri/target/release/bundle/nsis/VoxType_0.8.3_x64-setup.exe`

**Interfaces:**
- Produces: Git commit、远端 `main`、`v0.8.3` 标签、Latest GitHub Release 和 NSIS 安装包资产。

- [x] **Step 1: 运行完整日常与发布检查**

Run: `npm run ai:check`

Expected: 11 项本地阶段、22 项 Vitest 和 235 项 Rust 测试全部通过。

Run: `npm run ai:release-check`

Expected: 5 项发布阶段、npm/Rust 审计、Clippy 和 Tauri debug 构建全部通过；仅允许项目已记录的 17 条上游 Rust audit warning。

- [x] **Step 2: 构建安装包并记录校验值**

Run: `npm run tauri -- build`

Expected: 生成 `src-tauri/target/release/bundle/nsis/VoxType_0.8.3_x64-setup.exe`。

Run: `$installer = Resolve-Path 'src-tauri/target/release/bundle/nsis/VoxType_0.8.3_x64-setup.exe'; Get-Item -LiteralPath $installer; Get-FileHash -LiteralPath $installer -Algorithm SHA256`

Expected: 文件存在且大小大于 0，SHA-256 为 64 位十六进制字符串；实际值写入发布审计。

- [x] **Step 3: 审计定稿后重新验证并检查提交范围**

Run: `npm run ai:check`

Expected: 最终工作树内容通过。

Run: `git diff --check`、`git status --short`、`git diff --stat`、`npm run scan:secrets:staged`

Expected: 只有计划内源码、测试、文档和版本文件，无密钥、识别正文或无关改动。

- [ ] **Step 4: 提交、推送、打标签并创建 Release**

Commit message: `发布 0.8.3 修复 FunASR 累计字幕`

推送 `main`，创建并推送 annotated tag `v0.8.3`，使用 GitHub CLI 创建名为 `VoxType 0.8.3` 的正式 Latest Release并上传 `VoxType_0.8.3_x64-setup.exe`。

- [ ] **Step 5: 验证远端发布和 CI**

Run: `gh release view v0.8.3 --json name,tagName,isDraft,isPrerelease,assets,url,publishedAt`

Expected: 非草稿、非预发布、Latest Release，资产大小和 digest 与本地一致；远端 `main`、标签提交与本地 HEAD 一致；该提交触发的 GitHub Actions 结束且结论为 success。
