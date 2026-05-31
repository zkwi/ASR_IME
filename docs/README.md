# VoxType 文档索引

本目录保存 VoxType 的工程文档、Wiki 草稿、发布审计和外部接口参考。项目文档优先服务实际维护，避免为了完整而重复写多份同样内容。

## 用户文档

- 主要入口：[README.md](../README.md) / [README.en.md](../README.en.md)
- Wiki 首页草稿：[docs/wiki/Home.md](wiki/Home.md)
- 配置指南：[docs/wiki/Setup-Guide.md](wiki/Setup-Guide.md) / [docs/wiki/Setup-Guide-English.md](wiki/Setup-Guide-English.md)
- 功能与使用优化：[docs/wiki/Feature-Guide.md](wiki/Feature-Guide.md) / [docs/wiki/Feature-Guide-English.md](wiki/Feature-Guide-English.md)
- 常见问题与排障：[docs/wiki/Troubleshooting.md](wiki/Troubleshooting.md) / [docs/wiki/Troubleshooting-English.md](wiki/Troubleshooting-English.md)

## 维护文档

- AI 和维护规则：[AGENTS.md](../AGENTS.md)
- 架构概览：[docs/architecture.md](architecture.md)
- 代码规范：[docs/code-style.md](code-style.md)
- 目录结构规范：[docs/directory-structure.md](directory-structure.md)
- 贡献指南：[CONTRIBUTING.md](../CONTRIBUTING.md)
- 支持说明：[SUPPORT.md](../SUPPORT.md)
- 安全策略：[SECURITY.md](../SECURITY.md)

## 发布与参考

- 更新日志：[CHANGELOG.md](../CHANGELOG.md)
- 发布审计记录：[docs/audits/](audits/)
- LLM 润色模型测试：[2026-05-28 LLM 润色模型测试记录](audits/2026-05-28-llm-polishing-model-test.md)，含 2026-05-30 基于当前简化 prompt 和 `thinking_strategy` 的修正复测。
- LLM 文档同步审计：[2026-05-30 LLM 文档同步审计](audits/2026-05-30-llm-docs-sync-audit.md)
- LLM Prompt 拼装优化审计：[2026-05-30 LLM Prompt 拼装优化审计](audits/2026-05-30-llm-prompt-assembly-audit.md)
- LLM 长文本提示词优化审计：[2026-05-30 LLM 长文本提示词优化审计](audits/2026-05-30-llm-length-aware-prompt-audit.md)
- LLM 成稿化提示词复测审计：[2026-05-31 LLM 成稿化提示词复测审计](audits/2026-05-31-llm-editorial-prompt-audit.md)
- 设计和优化计划：[docs/plans/](plans/)
- 豆包流式 ASR 参考：[docs/豆包流式语音识别参考文档.md](豆包流式语音识别参考文档.md)

## 同步规则

- 用户可见行为变化：同步 README、英文 README、相关 Wiki 草稿和必要的线上 Wiki。
- 配置字段变化：同步 Rust 默认值、配置模板、前端设置项、三语言文案、README 和 Wiki 配置指南。
- 排障流程变化：同步 SUPPORT、Troubleshooting 草稿和 README 中的关键入口。
- 隐私、安全或日志脱敏变化：同步 SECURITY、README 和相关 Wiki 段落。
- 截图更新：放在 `screenshots/`，截图中不得出现真实密钥、识别正文、屏幕 OCR 正文、个人热词、prompt、最近上下文或 Windows 用户名路径。
- 历史审计记录只修正明显错字或敏感信息，不为追求一致性改写当时结论。

## 自动检查

日常本地检查会运行：

```powershell
npm run ai:check
```

文档和工程治理检查可单独运行：

```powershell
npm run check:governance
```

该检查会验证版本号一致性、本地 Markdown 链接、截图引用、README 中 GitHub Wiki 链接对应的 `docs/wiki/` 草稿是否存在。

治理检查脚本自身的最小回归测试：

```powershell
npm run test:governance
```
