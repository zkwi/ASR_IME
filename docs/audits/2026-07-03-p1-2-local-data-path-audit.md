# P1-2 安装版本地数据路径审计

日期：2026-07-03

## 改动范围

- 安装版配置默认路径改为 `%APPDATA%\VoxType\config.toml`。
- 安装版日志默认路径改为 `%LOCALAPPDATA%\VoxType\logs\voice_input.log`。
- 开发模式继续使用仓库根目录的 `config.toml` 和 `voice_input.log`，保留本地调试便利性。
- 安装版启动时如发现旧位置已有 VoxType `config.toml`，且新默认位置不存在配置，会在主窗口询问是否复制迁移。
- 隐私与本地数据页显示后端返回的当前实际配置和日志路径。

## 设计取舍

- 迁移只复制旧配置到新默认位置，不删除旧文件，不做多版本备份，符合个人项目的简单策略。
- 旧配置候选只接受包含 `[auth]`、`[request]`、`[audio]`、`app_key`、`access_key` 或 `ws_url` 的文件，降低误把其他 `config.toml` 当作 VoxType 配置的概率。
- 用户拒绝迁移后，前端用本地 `localStorage` 记录一次，不在后续启动中反复提示同一组源/目标路径。

## 主链路审计

- 不修改 ASR 请求、音频采集、LLM 润色、剪贴板、热键、托盘、统计记录或成功/失败态判定。
- `load_config()` 和 `save_config()` 仍走同一套配置校验与迁移逻辑，只改变安装版默认文件位置。
- 最近上下文继续存放在配置文件旁的 `context/recent_context.jsonl`，且不会写回 `config.toml`。
- 日志仍经过既有脱敏逻辑；新增迁移日志不包含源路径、目标路径、密钥、热词、prompt、最近上下文或识别正文。

## 验证

- `npm run check`：通过，0 errors / 0 warnings。
- `cargo fmt`：通过。
- `cargo test config::tests`：通过，23 个测试通过。
- `cargo test app_log::tests`：通过，4 个测试通过。
- `cargo check`：通过。

## 手工回归建议

- 开发模式从仓库目录启动，确认仍读取/保存仓库根目录 `config.toml`。
- 安装版首次启动且 `%APPDATA%\VoxType\config.toml` 不存在时，确认会生成/保存到新默认位置。
- 安装版发现 exe 旁或旧工作目录存在 VoxType `config.toml` 时，确认主窗口出现迁移确认；确认后新位置出现复制后的配置。
- 用户拒绝迁移后，确认同一旧配置不会每次启动都重复提示。
- 隐私与本地数据页确认显示实际配置路径和日志路径。
