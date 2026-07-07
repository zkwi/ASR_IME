# VoxType 代码规范

本规范用于约束 AI 和开发者的日常改动。目标是保持代码简单、可读、稳定，不追求过度工程化。

---

## 1. 总原则

1. 优先保持现有架构，不做无必要重构。
2. 一个改动只解决一个问题。
3. 用户可见行为必须有清楚文案。
4. 配置、日志、隐私、剪贴板、热键属于高风险区域，修改时必须说明影响。
5. 新增依赖前必须说明必要性；能不用依赖就不用。
6. 代码应直接、清楚，避免为个人项目引入复杂抽象层。

---

## 2. Rust 代码规范

1. Rust 代码必须通过：

```powershell
Set-Location .\src-tauri
cargo fmt --check
cargo check
cargo test
```

2. 模块职责保持清楚：

- `session.rs` 只处理会话状态和启动/停止流程。
- `audio.rs` 只处理音频采集。
- `asr_ws/` 只处理豆包 WebSocket 会话、音频发送、最终结果流转和错误映射。
- `llm_post_edit.rs` 只处理 LLM 润色判断和调用。
- `text_output.rs` 只处理剪贴板和粘贴。
- `hotkey.rs` 只处理热键和输入钩子。
- `config.rs` 只处理配置模型、加载、保存和校验。

3. 不要在多个模块重复实现同一逻辑。
4. 用户可见错误不要直接返回底层错误字符串，应转换成用户能理解的提示。
5. 日志必须脱敏，不得记录真实密钥、识别正文或最近上下文正文。
6. 统计只记录非正文数据，例如字数、时长、速度、错误码。
7. `text_output.rs` 中直接接触 Win32 剪贴板和全局内存的代码应优先通过 RAII 小封装管理 `OpenClipboard`、`GlobalAlloc` 和 `GlobalLock` 生命周期。`SetClipboardData` 成功后才转移内存所有权。

---

## 3. Svelte / TypeScript 代码规范

1. 前端代码必须通过：

```powershell
npm run check
npm run build
```

2. `src/routes/+page.svelte` 当前仍是主页面入口，可以小改，但不要继续新增大型复杂逻辑。
3. 新增可复用 UI 应放到：

```text
src/lib/components/
```

4. 新增纯函数、格式化函数、状态映射函数应放到：

```text
src/lib/utils/
```

5. 用户可见文案应放到：

```text
src/lib/i18n/
```

6. 三语言必须同步：

```text
zh-CN.ts
zh-TW.ts
en.ts
```

7. 首页不显示调试路径、协议细节、内部状态码。
8. 设置页优先展示用户可理解的基础和低频项；协议、快照、重试等底层参数不放入 UI，保留在 `config.toml`。
9. 悬浮字幕不显示瞬时成功状态；普通成功输出完成后应立即隐藏，仅错误或需要用户注意的告警可短暂停留。
10. `VoxTypeController.svelte.ts` 应作为前端组合入口使用。新增功能优先放入 `src/lib/app/*Controller.svelte.ts` 或 `src/lib/utils/`，不要把新的业务状态继续堆进主 controller。

---

## 4. 配置规范

新增配置项必须同步：

1. Rust 配置模型。
2. Rust 默认值。
3. 配置校验。
4. `config.example.toml`。
5. 前端设置页。
6. 三语言文案。
7. README 和 README.en.md 必要说明。
8. `docs/wiki/Setup-Guide*.md` 中的配置说明。
9. 字段校验逻辑和默认值说明。

默认值必须保守：

- 最近上下文默认关闭。
- 右 Alt 默认关闭。
- 鼠标中键默认关闭。
- 录音期间系统静音默认关闭。
- 不默认打印识别正文。

---

## 5. 文案规范

用户文案应做到：

1. 短。
2. 明确。
3. 告诉用户下一步怎么做。
4. 不暴露内部实现。
5. 错误提示必须包含原因和操作建议。

示例：

推荐：

```text
没有识别到文字。请确认麦克风有声音，或靠近一点再试。
```

不推荐：

```text
ASR final result empty.
```

---

## 6. 剪贴板规范

剪贴板相关改动必须遵守：

1. 写入失败必须返回错误或 warning。
2. 自动粘贴失败时，不得丢失识别文本。
3. 原剪贴板恢复失败时，必须给出 warning。
4. 不得承诺无法保证的恢复能力。
5. 仅复制模式不得发送粘贴快捷键。
6. 粘贴成功与否不能仅依赖按键发送成功判断。

---

## 7. 热键规范

热键相关改动必须遵守：

1. 主热键默认开启。
2. 右 Alt 默认关闭。
3. 鼠标中键默认关闭。
4. 配置读取失败时必须回落到保守触发策略。
5. 鼠标中键会占用其他软件中键行为，UI 必须提示风险。
6. 修改热键保存逻辑时，应尽可能测试热键是否可注册。

---

## 8. 提交前检查

日常提交前运行：

```powershell
npm run ai:check
```

文档、版本号和 Wiki 镜像一致性可单独运行：

```powershell
npm run check:governance
```

统计展示口径可单独回归：

```powershell
npm run test:stats
```

发布前可运行：

```powershell
npm run ai:release-check
```

如果不能运行，提交说明中必须写明原因。

---

## 9. 版本号策略

发布时不要机械地只递增 patch。版本号需要表达这次发布对用户的实际价值：

1. 小修复、文档、小范围文案和纯维护改动，用 patch。
2. 用户明显能感知的新功能、默认策略调整、性能或识别体验优化、设置项扩展，优先升 minor。
3. 破坏兼容的配置、数据、安装路径或核心使用方式变化，升 major，或至少在发布说明里明确破坏性影响。

如果一轮迭代已经包含多项用户可见体验优化，可以直接从 `0.1.x` 跳到 `0.2.0`，不要因为项目仍是个人项目就长期压低版本号。版本号调整仍需保持 `package.json`、`package-lock.json`、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock`、`src-tauri/tauri.conf.json`、`CHANGELOG.md` 和发布审计记录一致。
