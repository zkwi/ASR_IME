# VoxType full audit - 2026-05-04

本记录用于保存一次完整项目健康审计的非敏感结论。不要在审计记录中写入真实密钥、识别正文、热词、prompt、最近上下文、日志正文或本机用户名路径。

## 范围

- Git 与 GitHub 状态：本地分支、远端分支、开放 PR、Wiki 仓库状态。
- 依赖安全：npm audit、cargo audit、CI/发布前检查。
- 密钥与本地状态文件：Git 跟踪文件、显式密钥模式、项目密钥扫描脚本。
- 主链路保护：空识别、剪贴板恢复、统计正文、日志脱敏、保守默认值。
- 命令执行面：注册表开机启动、更新安装器启动。
- 文档一致性：README、Wiki 草稿、关键默认值说明。

## 结论

- 远端仓库只保留 `main` 分支，当前没有开放 PR。
- 未发现被 Git 跟踪的本地配置、日志、统计、最近上下文或自动热词历史文件。
- 未发现真实密钥。手写密钥模式扫描只命中测试里的假 key 和单元测试占位值。
- npm audit 为 0 vulnerabilities。
- cargo audit 通过，但报告 19 条上游依赖 warning，主要来自 Tauri/wry 间接引入的 Linux GTK 生态未维护 warning，以及 `glib`、`rand` 间接依赖 warning；当前项目是 Windows 桌面应用，未直接使用这些 crate。继续通过 Dependabot 和 CI 追踪。
- 主链路默认值符合 `AGENTS.md`：最近上下文、右 Alt、鼠标中键、录音期间静音系统声音默认关闭；自动粘贴后默认恢复剪贴板；统计不记录识别正文；日志和诊断报告有脱敏测试覆盖。
- 命令执行面未发现 shell 拼接用户输入。开机启动使用固定 `reg` 命令和参数数组；更新安装器启动使用已下载的安装包路径和固定静默参数。
- README、Wiki 草稿与关键默认值一致：连续低音量自动停止默认 30 秒，阈值 `0.03`；剪贴板恢复延迟默认 800ms。

## 已运行检查

```powershell
git status --short --branch
gh pr list --state open --json number,title,state,headRefName,url
git ls-remote --heads origin
git ls-files | rg "(^|/)(config\.toml|voice_input\.log|voice_input_stats\.jsonl|recent_context\.jsonl|hotword_history\.jsonl|\.env($|\.))"
npm run scan:secrets
npm run test:secrets
npm run audit:npm
npm run audit:rust
npm run ai:release-check
```

`npm run ai:release-check` 内部已覆盖：

```text
npm run check
npm run build
npm run scan:secrets
npm run test:secrets
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
npx tauri build --debug --no-bundle
```

## 后续建议

- 下次处理 Rust 依赖更新时，不要直接合并包含 `windows` crate 大版本变化的 Dependabot 组合 PR；先拆分或确认 `HWND` 类型来源一致。
- 继续保留 `npm run ai:check` 作为日常提交前入口，发布前使用 `npm run ai:release-check`。
- 若未来启用更多 Linux/macOS 构建目标，应重新评估 cargo audit 中 GTK/wry 间接依赖 warning 的实际影响。
