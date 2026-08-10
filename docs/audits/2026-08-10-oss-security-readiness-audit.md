# VoxType OSS 与安全就绪审计

日期：2026-08-10

## 结论

本轮未发现可直接复现的真实密钥、识别正文或本机用户名路径泄漏，也未发现前端不安全 HTML 注入、shell 字符串拼接执行或绕过最终 ASR 包门禁的新问题。已修复两个有明确上游修复版本的依赖风险，补齐公开 CI、CodeQL、私有漏洞报告和 Dependabot 安全更新，并将更新安装包缺少独立可信校验列为公开安全后续工作。

本轮定位为 OSS 治理与发布整理，不修改录音、ASR、LLM、剪贴板、热键、托盘、配置字段或默认值。唯一 Rust 代码变更是移除一个依赖 Windows runner 调度速度的测试上界，生产逻辑未变。

## 审计范围与方法

- 阅读主链路、配置、日志、诊断、统计、最近上下文、自动热词历史、更新器、Tauri CSP/Capability、GitHub Actions 和发布脚本。
- 搜索 Rust `unsafe`、进程启动、命令参数、日志写入、下载与安装路径；搜索前端 `innerHTML`、Svelte `{@html}`、`eval` 和 `new Function` 等动态执行入口。
- 运行仓库密钥扫描及其自测、治理检查及其自测、前端类型检查与构建、Vitest、Rust fmt/check/test、npm audit、RustSec audit 和定向回归。
- 人工检查仓库中的 5 张 PNG 截图；未发现可读真实密钥、识别正文或 Windows 用户名路径。
- 复核 GitHub 仓库安全设置、现有 Dependabot 配置、Actions 权限和第三方 Action 引用。

## 已修复问题

### 1. npm 间接依赖漏洞

- 审计前：`postcss` 间接依赖 `nanoid 3.3.16`，命中 `GHSA-2v37-7h3g-55p8`，`npm audit` 为 high。
- 处理：仅更新 `package-lock.json`，将 `nanoid` 升到 `3.3.18`；未增加或改动直接依赖。
- 结果：`npm audit` 为 0 个已知漏洞，`npm ls nanoid` 确认为 `3.3.18`。

### 2. Rust 间接依赖 unsoundness

- 审计前：`event-listener 5.4.1` 命中 `RUSTSEC-2026-0221`，上游修复版本为 `5.4.2`。
- 处理：仅更新 `src-tauri/Cargo.lock` 到 `event-listener 5.4.2`。
- 结果：RustSec 不再报告该 advisory；全量 Rust 测试和 clippy 继续作为发布门禁。

### 3. Windows CI 调度型波动

- 远端 `v0.11.0` CI 的 248 项 Rust 测试中仅 `screen_context::tests::wait_for_context_times_out_without_result` 失败。函数已在 1ms 超时后正确返回 `None`，失败来自测试额外要求整个调用必须在 100ms 内结束。
- 处理：保留真实 1ms 超时断言，删除无法由进程保证的 wall-clock 上界；未修改生产实现。
- 结果：定向测试连续运行 20 次全部通过，全量 248 项 Rust 测试通过。

## 隐私与本地数据结论

- `stats.rs` 只持久化时间、时长和字数；识别文本仅在内存中用于计数，不写入统计事件。
- 最近上下文和自动热词历史默认关闭，正文写入独立本地 `context/` 文件，不写回 `config.toml`；开发目录对应内容被 `.gitignore` 排除。
- 日志写入统一经过换行压平、长度限制、常见 key/token/bearer/password/secret 形态、`sk-`、`ark-` 和当前 `%USERPROFILE%` 路径脱敏。
- 诊断报告只输出布尔状态、计数、会话阶段和错误码；不包含正文、OCR、热词、Prompt、最近上下文正文、自动热词正文或密钥。
- 实时字幕和中间 ASR 文本仍只作显示反馈；最终包、空结果、LLM 资格、剪贴板写入和成功统计门禁均由现有 248 项 Rust 回归覆盖。

## 应用与仓库安全结论

- Tauri 配置存在显式 CSP：默认仅自身资源，IPC 连接受限，图片仅允许自身/blob/data，样式允许现有内联样式；前端未发现动态 HTML 或动态代码执行入口。
- Windows `unsafe` 代码集中在热键、剪贴板、系统音频、屏幕捕获和悬浮窗集成模块，没有扩散到网络或配置解析层。
- 外部进程使用参数数组调用 `reg` 或已下载的安装器，未发现把用户输入拼成 shell 命令的路径。
- GitHub Actions 顶层权限为只读内容；CodeQL 仅增加上传安全结果所需的 `security-events: write`。第三方 Action 固定到完整 commit SHA，并由 Dependabot 跟踪。
- GitHub Secret Scanning、Push Protection、Dependabot security updates 和 Private Vulnerability Reporting 已启用。

## 未关闭风险与处置

### 更新安装包独立校验（需设计，已跟踪）

当前更新器从配置的 GitHub 仓库读取最新 Release，经 HTTPS 下载优先级最高的 Windows `.exe` 后静默启动。它尚未使用独立信任锚验证 Authenticode、签名清单、摘要或 provenance；仓库或发布凭据失陷时影响较高。

本轮不在没有密钥管理、兼容策略和失败 UX 设计的情况下临时硬塞校验。公开 Issue [#34](https://github.com/zkwi/VoxType/issues/34) 已要求先完成威胁模型、信任锚、发布端与客户端流程、轮换恢复和回归计划，再单独实现。

### RustSec 上游状态 warning（接受并持续监控）

锁文件仍报告 17 条允许级 warning：GTK3/Linux Tauri 依赖链的未维护或 unsound 状态，以及 `tauri-utils` 经 `urlpattern` 引入的若干未维护 `unic-*` crate。`cargo tree --target x86_64-pc-windows-msvc -i glib` 为空，说明 GTK/glib 链不进入当前 Windows 构建；`unic-*` 为上游维护状态警告，当前无可利用漏洞结论。继续由 Cargo Dependabot、RustSec 和 Tauri 升级跟踪，不用大范围强制升级制造发布风险。

### Capability 最小权限（低风险观察项）

当前 `opener:default` 与主窗口、字幕窗、启动提示窗共用同一 Capability。未发现辅助窗口可控 HTML 或直接调用外部打开功能的利用路径，但从最小权限角度仍有收窄空间。只有在真实 Tauri 窗口回归能覆盖三类窗口时再拆分 Capability，避免为了审计分数破坏启动或字幕行为。

## 验证证据

- `npm run ai:check`：通过；Svelte 0 errors/0 warnings，生产构建成功，14 个 Vitest 文件共 61 项通过，Rust 248 项通过。
- `npm run audit:npm`：通过，0 vulnerabilities。
- `npm run audit:rust`：通过；可修复的 `RUSTSEC-2026-0221` 已消失，剩余 17 条上游状态 warning 如上记录。
- OCR 超时定向测试：连续 20/20 次通过。
- YAML 解析、`cargo fmt --check`、`git diff --check`、治理检查和密钥扫描：通过。
- CodeQL、拆分后的远端 CI、正式安装包、SHA-256 和 GitHub Release 状态在 `0.12.0` 发布审计中记录。
