# VoxType 目录结构规范

本文件定义 VoxType 后续新增文件应放在哪里。目标是保持结构简单、清楚、易维护。

---

## 1. 当前顶层目录

```text
VoxType/
├── src/                 # Svelte 前端
├── src-tauri/           # Tauri / Rust 后端
├── docs/                # 文档
├── scripts/             # 本地脚本
├── static/              # 静态资源
├── screenshots/         # README、Wiki 或发布说明使用的截图
├── config.example.toml  # 配置模板
├── package.json
├── svelte.config.js
├── tsconfig.json
├── vite.config.js
└── AGENTS.md            # AI 维护规则
```

---

## 2. 前端目录规范

推荐结构：

```text
src/
├── routes/
│   └── +page.svelte
└── lib/
    ├── components/
    │   ├── overview/
    │   ├── settings/
    │   ├── overlay/
    │   └── common/
    ├── i18n/
    ├── utils/
    └── types/
```

### 规则

1. `src/routes/+page.svelte` 是主入口，短期保留。
2. 新增组件放到 `src/lib/components/`。
3. 首页组件放到 `src/lib/components/overview/`。
4. 设置页组件放到 `src/lib/components/settings/`。
5. 悬浮字幕组件放到 `src/lib/components/overlay/`。
6. 通用按钮、提示、卡片放到 `src/lib/components/common/`。
7. 三语言文案放到 `src/lib/i18n/`。
8. 格式化函数、状态映射函数放到 `src/lib/utils/`。
9. 共享类型放到 `src/lib/types/`。

### 禁止

1. 不要在 `+page.svelte` 继续新增大型业务块。
2. 不要在组件里硬编码大量用户文案。
3. 不要为一个只用一次的小逻辑创建过深目录。
4. 不要新增 `src/components`、`src/common` 等与 `src/lib/components` 重复的目录。

---

## 3. Rust / Tauri 目录规范

当前结构：

```text
src-tauri/
├── src/
│   ├── app_log.rs
│   ├── asr.rs
│   ├── asr_ws.rs
│   ├── audio.rs
│   ├── autostart.rs
│   ├── config.rs
│   ├── hotkey.rs
│   ├── llm_post_edit.rs
│   ├── overlay.rs
│   ├── protocol.rs
│   ├── session.rs
│   ├── setup_guide.rs
│   ├── stats.rs
│   ├── system_audio.rs
│   ├── text_output.rs
│   ├── tray.rs
│   ├── update.rs
│   └── lib.rs
├── capabilities/
├── icons/
├── Cargo.toml
└── tauri.conf.json
```

### 模块职责

1. `session.rs`：录音会话状态。
2. `audio.rs`：麦克风采集。
3. `asr.rs`：ASR 请求组装、上下文和结果解析。
4. `asr_ws.rs`：ASR WebSocket 会话。
5. `protocol.rs`：豆包二进制协议。
6. `llm_post_edit.rs`：LLM 润色。
7. `text_output.rs`：剪贴板和自动粘贴。
8. `hotkey.rs`：全局热键和输入钩子。
9. `overlay.rs`：悬浮字幕。
10. `tray.rs`：系统托盘。
11. `config.rs`：配置模型、加载、保存、校验。
12. `app_log.rs`：日志和脱敏。
13. `stats.rs`：使用统计。
14. `update.rs`：更新检查。
15. `system_audio.rs`：系统音量。
16. `autostart.rs`：开机启动。

### 规则

1. 新增 Rust 模块必须有明确职责。
2. 不要新建 `utils.rs` 这种大杂烩模块。
3. 不要让 `lib.rs` 承担大量业务逻辑。
4. 不要在 `asr_ws.rs` 中直接处理 UI 复杂逻辑。
5. 不要在 `text_output.rs` 之外写剪贴板逻辑。
6. 不要在 `hotkey.rs` 之外写全局热键或输入钩子逻辑。
7. 不要在 `stats.rs` 中保存识别正文。
8. 不要在 `app_log.rs` 外绕过日志脱敏。

---

## 4. 文档目录规范

```text
docs/
├── code-style.md
├── directory-structure.md
├── setup-guide.md
└── ...
```

### 规则

1. 面向用户的说明优先放 README。
2. 面向开发和 AI 的规则放 docs。
3. 接口参考文档可以放 docs，但不要混入真实密钥。
4. 大改功能时，如果影响维护规则，应同步更新 docs。

---

## 5. 脚本目录规范

```text
scripts/
├── ai-check.ps1
├── ai-release-check.ps1
├── enable_git_hooks.ps1
└── scan-secrets.mjs
```

### 规则

1. 本地检查脚本放 scripts。
2. 脚本应尽量可读，不要做过度复杂的环境探测。
3. 脚本失败应返回非零退出码。
4. 涉及发布的脚本必须避免打包本地配置、日志、统计和上下文。

---

## 6. 禁止新增的目录

除非有明确理由，不要新增：

```text
src/components/
src/common/
src/helpers/
src-tauri/src/utils.rs
src-tauri/src/common.rs
tmp/
temp/
build-output/
release-assets/
```

---

## 7. 新文件放置决策

新增文件前先判断：

```text
是用户界面组件？ → src/lib/components/
是前端纯函数？ → src/lib/utils/
是前端类型？ → src/lib/types/
是用户文案？ → src/lib/i18n/
是 Rust 桌面能力？ → src-tauri/src/对应模块
是开发规范？ → docs/
是本地脚本？ → scripts/
是用户说明？ → README.md
```
