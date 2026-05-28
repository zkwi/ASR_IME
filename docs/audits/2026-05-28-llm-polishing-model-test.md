# 2026-05-28 LLM 润色模型测试记录

本记录整理一次面向 VoxType 语音输入润色场景的 DashScope OpenAI 兼容 API 模型测试结果，用于选择日常润色模型和排查模型权限、延迟问题。

## 测试边界

- 测试日期：2026-05-28。
- 测试 API：阿里云百炼 DashScope OpenAI 兼容 API。
- 接口地址：`https://dashscope.aliyuncs.com/compatible-mode/v1`。
- 调用方式：`/chat/completions`。
- 本次测试基于用户当前 VoxType 配置，只替换请求中的模型名，没有修改本地 `config.toml`。
- 本记录不包含真实 API Key、真实识别正文、本地最近上下文、热词、场景偏好、日志或统计正文，也不复述实际 prompt 内容。

## 测试配置

- `enable_thinking = false`
- 超时：30 秒
- 使用 VoxType 当前大模型 System Prompt 和 User Prompt Template
- 主要目标：评估语音输入润色的效果、稳定性和速度

## 测试样例类型

共使用 8 类合成语音输入文本：

1. 口语清理与状态规则
2. 技术词、路径、API Key、日志
3. 金融数字表达
4. 中英混合表达
5. 问题只润色、不回答
6. 提示词注入文本只当作待润色内容
7. 会议事项整理
8. 剪贴板与隐私说明

## 结果概览

| 模型 | 成功率 | 平均耗时 | 最慢一次 | 评价 |
| --- | ---: | ---: | ---: | --- |
| `qwen3.6-flash-2026-04-16` | 8/8 | 611 ms | 772 ms | 最快，日常可用，但技术路径易改错 |
| `deepseek-v4-flash` | 8/8 | 904 ms | 1246 ms | 较快，格式细节一般 |
| `qwen3.7-max` | 8/8 | 1232 ms | 1826 ms | 综合最好，推荐日常优先使用 |
| `deepseek-v4-pro` | 8/8 | 1222 ms | 1463 ms | 技术文本保真较好，中文润色略弱 |
| `qwen3.6-plus` | 8/8 | 1966 ms | 6097 ms | 效果好，但有明显慢请求 |
| `glm-5.1` | 8/8 | 2102 ms | 2438 ms | 稳定但润色偏保守 |
| `qwen3.6-max-preview` | 8/8 | 2337 ms | 3355 ms | 慢，效果没有明显优势 |
| `kimi-k2.6` | 8/8 | 2839 ms | 7358 ms | 可用但波动大，不适合实时润色默认选择 |
| `qwen3.7-max-preview` | 当前配置不可用 | - | - | 要求 `enable_thinking = true` |

`qwen3.7-max-preview` 在当前 `enable_thinking = false` 配置下返回 400，提示必须开启 `enable_thinking = true`。单独开启后可用，但 3 组样例耗时约 13.8 到 24.5 秒，不适合作为实时润色默认选择。

## 关键结论

`qwen3.7-max` 是这轮测试的综合最优选择。它在中文润色、标点整理、中英混合、隐私规则和提示词注入样例中表现最均衡，速度也能接受。

`qwen3.6-flash-2026-04-16` 延迟最低，适合追求即时反馈的语音输入场景。但它在技术路径上容易把 `src-tauri/src/llm_post_edit.rs` 这类内容改错，技术文本较多时需要谨慎。

`deepseek-v4-pro` 在技术路径保留上表现最好，但整体语句润色自然度不如 `qwen3.7-max`。

`glm-5.1` 输出稳定，但偏保守，很多句子只是轻度整理。

`kimi-k2.6` 润色效果可以，但延迟波动明显，首个样例耗时 7358 ms，不适合作为实时语音输入默认选择。

## 推荐用法

日常默认优先考虑：

```toml
model = "qwen3.7-max"
```

低延迟优先时可考虑：

```toml
model = "qwen3.6-flash-2026-04-16"
```

技术路径、代码文件名和日志文本较多时可考虑：

```toml
model = "deepseek-v4-pro"
```

不建议作为 VoxType 实时润色默认选择：

```text
qwen3.7-max-preview
qwen3.6-max-preview
kimi-k2.6
```

切换模型前，先确认该模型对当前 DashScope 账号和地域可用。切换后建议在 API 配置页重新点击大模型测试，确认权限、网络和延迟都符合预期。
