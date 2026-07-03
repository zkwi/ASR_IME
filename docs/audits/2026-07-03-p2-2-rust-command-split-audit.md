# P2-2 Rust 命令入口拆分审计

日期：2026-07-03

## 变更范围

- 新增 `src-tauri/src/commands/`，按职责拆分 Tauri command：
  - `config_commands.rs`：配置加载/保存、初始化检查、ASR/LLM/OCR 测试、配置指南、托盘语言和关闭偏好。
  - `diagnostic_commands.rs`：日志、诊断报告、本地数据状态、统计清理、最近上下文、自动热词候选。
  - `update_commands.rs`：更新检查与下载安装。
  - `session_commands.rs`：悬浮字幕、麦克风设备、录音会话控制。
- `src-tauri/src/lib.rs` 只保留应用启动、窗口关闭处理、invoke handler 注册和启动期自启动同步辅助。
- `CHANGELOG.md` 记录本轮工程治理变更。

## 行为审计

- 主链路：不改变录音、ASR、LLM 润色、剪贴板写入、自动粘贴、统计写入逻辑。
- 配置结构：不新增字段，不修改默认配置，不触发配置迁移。
- 隐私与日志：诊断报告仍不包含识别正文、OCR 正文、热词、Prompt、最近上下文正文、自动热词历史正文、候选词或密钥原文；用户路径脱敏逻辑保持原语义。
- 热键与托盘：保存配置后的热键刷新、必要时重启全局热键线程、托盘语言设置、关闭到托盘行为保持原实现。
- 测试入口：ASR、LLM、OCR 测试命令保持原日志和返回结构；LLM 测试成功继续记录耗时和 thinking strategy。

## 已验证

- `cargo fmt`
- `cargo check`
- `cargo test commands::`

## 后续发布前检查

- 继续运行 `npm run ai:check`。
- 发布前运行 `npm run ai:release-check`。
