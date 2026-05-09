# 支持说明

VoxType 是个人项目，不提供商业 SLA。维护优先级是主链路稳定、隐私边界、安装配置体验和明确可复现的问题。

## 优先使用这些入口

- 配置和首次使用：<https://github.com/zkwi/VoxType/wiki/Setup-Guide>
- 常见问题和排障：<https://github.com/zkwi/VoxType/wiki/Troubleshooting>
- 功能说明：<https://github.com/zkwi/VoxType/wiki/Feature-Guide>
- Bug 报告：使用 GitHub Issue 模板。
- 安全和隐私问题：参考 [SECURITY.md](SECURITY.md)。

## 提交问题前

- 先搜索已有 Issue 和 Wiki。
- 隐去真实 API Key、识别正文、个人热词、prompt、最近上下文、屏幕 OCR 文本、自动热词历史、日志全文和 Windows 用户名路径。
- 尽量说明 VoxType 版本、Windows 版本、安装方式、麦克风、目标粘贴应用和复现步骤。
- 如果是粘贴、热键、托盘或更新问题，请说明是否能稳定复现。

## 快速自查

- 首次配置或 API 测试失败：先看 [Setup Guide](https://github.com/zkwi/VoxType/wiki/Setup-Guide)。
- 快捷键无反应：确认 API配置页健康检查没有阻断项，并确认至少一种触发方式已启用。
- 识别成功但没粘贴：先在记事本测试，再手动按 `Ctrl + V` 验证文本是否已写入剪贴板。
- 启动白屏或卡在启动页：优先按 [Troubleshooting](https://github.com/zkwi/VoxType/wiki/Troubleshooting) 的 WebView2 修复步骤处理。
- 日志或诊断报告反馈前，请确认已脱敏。

## 不适合公开 Issue 的内容

- 真实密钥、完整日志、未脱敏诊断报告。
- 包含个人语音输入正文或客户敏感信息的截图。
- 与 VoxType 无关的系统、网络、账号或第三方服务问题。

维护者会尽量处理清晰、可复现、影响真实使用的问题；模糊需求可能会被要求补充信息或暂缓。
