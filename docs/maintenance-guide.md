# VoxType 维护指南

本文面向日常维护和后续小版本迭代。目标是帮助维护者快速判断一次改动应该落在哪个模块，以及哪些边界不能被顺手改掉。

## 改动前先定位链路

VoxType 的核心链路是：

```text
触发录音 -> 麦克风采集 -> ASR provider -> 可选 LLM 润色 -> 剪贴板输出 -> 统计与本地历史
```

开始改代码前先判断改动是否触碰以下区域：

- ASR provider、音频分片、最终包选择、空识别处理。
- LLM 触发条件、prompt 拼装、最近上下文或屏幕 OCR 参考。
- 剪贴板写入、自动粘贴、原剪贴板恢复。
- 日志、诊断报告、统计、本地正文历史。
- 热键、托盘、窗口关闭行为。

如果触碰这些区域，先看 [ASR 质量与延迟守门清单](asr-quality-latency-guardrails.md)，再决定测试范围。

## ASR provider 边界

`src-tauri/src/asr_provider.rs` 是统一入口，只负责三件事：

- 判断当前 provider。
- 做启动前配置门禁。
- 把录音会话参数转交给具体 provider。

豆包协议细节放在 `asr_ws.rs`，阿里云 FunASR 协议细节放在 `aliyun_asr.rs`。不要把 provider-specific WebSocket payload、事件解析或最终文本选择逻辑塞回 `asr_provider.rs`。

新增 provider 时，优先沿用当前轻量分发方式。只有 provider 数量和共享行为复杂到明显重复时，再考虑 trait 或更重的抽象。

## 最终文本门禁

实时字幕和最终文本必须分开处理：

- 豆包：中间包只更新字幕，最终输出等待最终包和二遍分句选择。
- 阿里云：`result-generated` 只更新字幕，最终输出必须等 `task-finished`。

任何中间文本都不能触发 LLM、粘贴、成功统计、最近上下文或自动热词历史。空最终文本必须进入失败态。

## 设置页维护

设置页的目标是普通用户可上手，高级参数可修复：

- 高频、必要字段直接展示。
- 低频、协议、兼容或排障字段优先折叠。
- 已启用、非默认或有校验错误的高级区要能自动展开。
- 字段校验跳转由 `src/lib/utils/settingsFields.ts` 维护，panel id 必须和组件里的 `id` 对齐。
- 复用折叠面板时使用 `src/lib/components/common/AdvancedSettings.svelte`，不要在页面里重复写同一套 DOM 和 CSS。

新增设置字段时，按 `AGENTS.md` 的配置同步清单执行，不要只改 Rust 或只改前端。

## 隐私和诊断

默认不得写入日志、诊断报告、发布审计或截图的内容：

- 真实密钥。
- 识别正文。
- 热词、prompt、最近上下文正文。
- 屏幕 OCR 正文。
- Windows 用户名路径。

统计只保存非正文指标。最近上下文和自动热词历史即使开启，也只能进入各自本地数据文件，不写回 `config.toml`。

## 发布前检查

日常改动：

```powershell
npm run ai:check
```

发布前：

```powershell
npm run ai:release-check
npx tauri build
```

`ai:release-check` 会覆盖日常检查、npm audit、Rust audit、clippy 和 Tauri debug build。GitHub Actions CI 复用同一入口；如果本地发布检查没过，不要推送发布分支。

发布版本号要反映影响范围：

- patch：纯维护、文档、小修复、小范围文案。
- minor：用户可见功能、明显体验调整、默认策略变化。
- major：破坏兼容或需要用户重新理解核心使用方式。

发布时同步 `package.json`、`package-lock.json`、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock`、`src-tauri/tauri.conf.json`、`CHANGELOG.md` 和 `docs/audits/`。
