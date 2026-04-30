# 贡献指南

感谢关注 VoxType。这个仓库是个人项目，维护优先级是实用、轻量、易维护。欢迎提交小而清晰的改进；涉及主链路、配置结构或隐私边界的改动，请先开 Issue 说明方案。

## 适合提交的改动

- 明确的 bug 修复。
- 文档、配置示例、排障说明和本地检查脚本优化。
- 小范围 UI/交互改进，并同步三语言文案。
- 针对已有行为的测试补充。

## 提交前请确认

- 已阅读根目录 `AGENTS.md` 中的主链路、隐私和配置同步规则。
- 没有提交真实 API Key、个人热词、最近上下文、日志或统计文件。
- 用户可见文案已同步 `src/lib/i18n/zh-CN.ts`、`src/lib/i18n/zh-TW.ts`、`src/lib/i18n/en.ts`。
- 修改配置字段时已同步 Rust 默认值、`config.example.toml`、前端设置项、三语言文案和 README。
- 修改用户可见行为时已同步 README 或 `docs/`。

## 本地开发

```powershell
npm install
npm run tauri dev
```

日常提交前建议运行：

```powershell
npm run ai:check
```

如果只想快速检查密钥误提交：

```powershell
npm run scan:secrets
npm run scan:secrets:staged
```

依赖审计可以单独运行：

```powershell
npm run audit:npm
```

可以启用本地 pre-commit 钩子：

```powershell
.\scripts\enable_git_hooks.ps1
```

## Pull Request 要求

- 保持改动聚焦，一个 PR 解决一个问题。
- 在 PR 描述中说明是否影响 ASR、LLM、剪贴板、热键、托盘、日志、统计或配置结构。
- 说明已运行的检查命令；无法运行时说明原因。
- 涉及 UI 时附上关键状态的手工验证结果。
- 不引入无必要依赖，不做顺手大重构。

## Commit 信息

推荐使用简短的英文前缀，便于浏览历史：

```text
fix: handle empty recognition state
docs: add setup troubleshooting notes
chore: tighten local secret scan
```
