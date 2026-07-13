# 2026-07-13 代码质量与架构改进规划

本规划基于对当前代码库(0.7.9 + 未提交改动)的全面审计,面向 codex 逐项执行。

## 执行结果（2026-07-13，VoxType 0.8.0）

- P0-1、P0-2、P0-3、P1-1、P1-2、P1-3 已完成并发布。
- P1-3 同时关闭了 `reqwest` 默认 native-tls 特性，豆包与阿里云均通过真实凭据 + 程序生成短静音包的 rustls 连接测试；未采集真实麦克风，因此原计划中的“各录一句并粘贴”未声称完成。
- P0-3 使用 Tauri CSP 对象配置并保留 `style-src` 资产注入例外；主窗口、IPC、更新检查和 WebView 控制台已验证。
- P2-1 与 P2-2 明确延后：没有缺陷驱动，且会扩大本次发布回归面，不符合个人项目的局部改动原则。
- 最终自动化基线为 234 项 Rust 测试和 21 项 Vitest；下文 231 项是规划制定时的快照，不应当作当前动态数量。

## 审计结论摘要

项目整体健康度**很高**,不存在需要"抢救"的技术债:

- 231 个 Rust 单元测试全部通过;svelte-check 0 错误 0 警告;前端无 `any` 类型。
- CI 跑发布级检查(clippy `-D warnings`、cargo audit、npm audit、密钥扫描、i18n 三语言 key 对齐、版本号一致性、文档链接检查)。
- 运行时代码几乎无 `unwrap()`(143 处集中在测试);无 TODO/FIXME 残留。
- 本地敏感文件(config.toml、统计、上下文)均正确 gitignore。
- 前端已完成控制器拆分,Rust 已完成 asr_ws/ 和 commands/ 拆分,模块边界与 docs/architecture.md 一致。

改进空间集中在:**重复代码收敛、测试基建现代化、少量安全加固、CI 提速**。以下按优先级排列。

---

## P0-1 收尾未提交的 dashscope thinking 策略改动（已完成）

**现状**:工作区有 `src-tauri/src/llm_request_adapter.rs` 的未提交改动(dashscope 平台 `omit` 策略改为强制 `enable_thinking`,候选列表去掉 `omit` 回退),测试已补且全部通过。

**任务**:
1. 确认该行为改动符合预期:dashscope 上配置为 `omit` 时不再按字面执行,而是转为 `dashscope_enable_thinking`;候选列表不再回退 `omit`。
2. 若确认,补一条 CHANGELOG `[未发布]` 条目(用户可见行为:dashscope 思考策略兼容性),然后提交。
3. 若不符合预期,回滚工作区。

**验收**:`cargo test` 通过;`git status` 干净;CHANGELOG 有对应条目。

---

## P0-2 收敛 LLM HTTP 客户端重复代码（已完成）

**现状**:`llm_post_edit.rs`(978 行)和 `hotword_generator.rs`(803 行)各自实现了一套几乎相同的:

- reqwest Client 构建(超时配置)
- `call_openai_compatible*` / `send_*_request`(thinking 策略候选重试循环)
- `chat_body` 组装
- 响应解析:`extract_message_content` 两处字面重复、`response_was_truncated`、错误友好化映射

provider 兼容性问题(thinking 策略)是当前活跃修改区,每次都要双份维护,遗漏风险高。

**任务**:
1. 新建 `src-tauri/src/llm_client.rs`,收敛以下共享能力(目标 ~150-200 行):
   - `build_client(timeout_seconds) -> reqwest::Client`
   - `send_chat_with_thinking_fallback(client, base_url, api_key, model, body, ...)`:封装 `thinking_strategy_candidates` 遍历 + `should_retry_without_unsupported_thinking` 重试循环
   - `extract_message_content` / `extract_reasoning_content` / `response_was_truncated`
2. `llm_post_edit.rs` 与 `hotword_generator.rs` 改为调用共享函数;各自保留 prompt 组装、参数校验、结果过滤等业务逻辑,不要过度抽象。
3. 现有单元测试全部保留并迁移到对应模块;不改变任何对外行为。

**约束**:遵守 AGENTS.md——一次只做这一件事,不顺手改 prompt 内容、触发条件或错误文案。

**验收**:`cargo test`、`cargo clippy --all-targets -- -D warnings` 通过;两个文件行数明显下降;`npm run ai:check` 通过。

---

## P0-3 设置 Tauri CSP(安全加固，已完成）

**现状**:`src-tauri/tauri.conf.json` 中 `"security": { "csp": null }`。前端为纯本地静态资源,风险可控,但设置 CSP 是 Tauri 官方推荐的低成本加固,对处理剪贴板和键盘注入的应用尤其值得。

**任务**:
1. 设置保守 CSP,例如:`"csp": "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'"`(Svelte 组件内联样式需要 `unsafe-inline`;Tauri 会自动为自身注入脚本加 nonce)。
2. 本地 `npx tauri dev` 和 `npx tauri build --debug --no-bundle` 后手工验证:主窗口、悬浮字幕、启动 toast、图标/截图显示、更新检查均正常,WebView 控制台无 CSP 报错。

**验收**:功能回归无异常;`npm run ai:release-check` 通过。

---

## P1-1 前端测试迁移到 vitest 并补齐纯函数测试（已完成）

**现状**:前端测试是三个手写 node 脚本(`test-overlay-layout.mjs`、`test-stats.mjs`、`test-governance.mjs`),每个脚本自己启动一个 Vite server 用 `ssrLoadModule` 加载 TS,再手写 assert。可用但难扩展。悬浮字幕在 0.7.3~0.7.8 连续 6 个版本修复,说明这类展示层纯逻辑恰恰最需要低摩擦的回归测试。同时 `setupStatus.ts`(222 行)、`appRouting.ts`(159 行)、`hotwords.ts`、`autoHotwords.ts`、`sessionState.ts` 等纯函数完全无测试。

**任务**:
1. 引入 `vitest`(唯一新增 devDependency,直接复用现有 vite 配置,删除自建的 ssrLoadModule 基建,净减自维护代码)。
2. 把 `test-overlay-layout.mjs`、`test-stats.mjs` 的断言原样迁移为 `src/lib/utils/*.test.ts`;`test-governance.mjs` 保持 node 脚本不动(它测的是治理脚本本身)。
3. 新增纯函数测试(优先级从高到低):`overlayLayout.ts` 边界补充、`setupStatus.ts`、`appRouting.ts`、`hotwords.ts`、`sessionState.ts`。只测纯函数,不引入组件测试/DOM 模拟。
4. `package.json` 增加 `"test:unit": "vitest run"`,并把它接入 `scripts/ai-check.ps1`,替换原 `test:overlay`/`test:stats` 入口;同步更新 AGENTS.md 第 7 节和 maintenance-guide.md 中的检查命令说明。

**验收**:`npm run test:unit` 通过;`npm run ai:check` 全绿;旧脚本删除后 CHANGELOG 记录工程治理条目。

---

## P1-2 CI 提速:缓存 Rust 构建与 cargo-audit（已完成）

**现状**:CI 每次 push 在 windows-latest 上冷编译整个 Tauri 项目并 `cargo install cargo-audit`(即使 `--locked` 也要现场编译),单次 30 分钟上限经常吃紧。

**任务**:
1. 在 `.github/workflows/ci.yml` 增加 `Swatinem/rust-cache@v2`(workspace 指向 `src-tauri`)。
2. cargo-audit 改用 `taiki-e/install-action@v2`(预编译二进制,秒级)或缓存 `~/.cargo/bin`。
3. 保持检查内容不变,只加缓存。

**验收**:CI 二次运行时间显著下降;检查项和当前完全一致。

---

## P1-3 统一 TLS 栈为 rustls（已完成，验收范围有说明）

**现状**:`reqwest` 用 `rustls-tls`,`tokio-tungstenite` 用 `native-tls`,二进制里同时链接两套 TLS,行为不一致且体积浪费。

**任务**:
1. `tokio-tungstenite` 特性从 `native-tls` 改为 `rustls-tls-webpki-roots`。
2. 手工验证豆包 WSS 和阿里云 FunASR WSS 均能建连、完成一轮完整识别(ASR 连接测试入口 + 实际录一句话)。

**约束**:这是主链路改动,若任一 provider 握手失败,立即回滚,不做适配 hack。

**验收**:两个 provider 连接测试通过 + 各完成一次真实录音粘贴;`cargo test` 通过。

---

## P2-1 config.rs 机械拆分(延后)

**现状**:`config.rs` 1732 行,混合了 19 个 struct 定义 + Default 实现、路径解析、旧配置迁移、加载/保存、recent context 文件 IO。AGENTS.md 禁止一次性大规模重构它,但纯移动式拆分风险低。

**任务**(单独一次提交,严格 move-only):
1. 拆为 `config/` 目录:`model.rs`(structs + Default)、`paths.rs`(路径解析 + 迁移候选)、`io.rs`(load/save + 字段迁移)、`recent_context.rs`(recent context 读写)。`config/mod.rs` re-export 全部现有公开项,**所有调用方 import 路径不变**。
2. 不改任何函数体、默认值、序列化行为;测试跟随各自代码移动。

**验收**:`git diff` 可确认只有移动;`cargo test`、`cargo clippy -D warnings`、`npm run ai:check` 通过;`cargo fmt --check` 通过。

**备注**:如果执行时发现无法做到纯移动(如私有可见性交叉),停下来汇报而不是顺手改逻辑。

---

## P2-2 VoxTypeController 瘦身(延后)

**现状**:`VoxTypeController.svelte.ts` 1320 行,是组合根,但仍残留一批可归位的领域函数(setup status 相关 ~10 个函数、asr 连接状态文案、mic 状态文案等),以及两个巨型 props 构建器。

**任务**(小步,每步独立可验证):
1. 把 `setupStatusItems`/`setupWarningCount`/`setupIsReady`/`handleSetupAction` 等 setup 相关函数移入 `setupController.svelte.ts`。
2. 把 `micStatusText`/`sidebarMicStatusText`/`currentAudioDevice` 等音频展示函数移入独立 util 或对应 controller。
3. **不要**重构 appShellProps/appContentProps 的显式 props 模式——它类型安全且被 svelte-check 全量校验,改成 context 得不偿失。

**验收**:`npm run check` 0 错误;主窗口手工回归(设置页、健康检查、录音一轮)。

---

## 明确不做的事项(已评估,避免 codex 自行发挥)

1. **不引入 thiserror/anyhow 重构错误类型**:136 处 `Result<_, String>` 是项目有意选择(见 `error.rs` 注释),Tauri command 边界用 String 足够,重构收益低、扰动大。
2. **不做 CSS 大规模去重**:HomeSection/AppShell 的大文件主体是组件私有 CSS(Svelte scoped),重复度可接受,统一化容易引入视觉回归。
3. **不为两个 ASR provider 抽象公共 trait**:豆包与阿里云的协议、上下文格式、门禁语义差异是真实的,当前轻量分发方式(asr_provider.rs)符合 maintenance-guide.md 约定,provider 到 3 家以上再考虑。
4. **不加端到端 mock WebSocket 集成测试**:收益与维护成本不成比例,现有 final_text/partial_text/errors 单测已覆盖协议关键分支。
5. **不动 docs/audits 组织方式**:append-only 审计记录是有效实践,92 个文件不构成问题。

## 本地环境顺手清理(非代码任务)

- `.codex/chrome-profile-hotwords/` 是一份完整 Chrome profile(约 3.8 万文件),已 gitignore,仅占本地磁盘,确认无用后可手动删除。

## 执行顺序建议

P0-1 → P0-2 → P1-1 → P0-3 → P1-2 → P1-3 → (按需) P2-1 → P2-2。
每项独立提交,遵守 AGENTS.md 的改动前七问、改动后汇报和 `npm run ai:check` 门禁;涉及用户可见行为的项(P0-1、P0-3)同步 CHANGELOG。
