# 2026-05-28 LLM 润色模型测试记录

本记录整理面向 VoxType 语音输入润色场景的模型测试结果，用于选择日常润色模型和排查模型权限、延迟问题。

> 2026-05-30 修正：5 月 28 日结论基于当时较长的默认 prompt。VoxType 0.1.84 之后默认 prompt 已简化，且 0.1.85 新增 `thinking_strategy` 思考适配；因此旧结论中“`deepseek-v4-pro` 更适合技术路径文本”不再作为当前推荐。当前推荐以本文 2026-05-30 复测为准。

## 测试边界

- 首次测试日期：2026-05-28。
- 修正复测日期：2026-05-30。
- 主要测试 API：阿里云百炼 DashScope OpenAI 兼容 API。
- 接口地址：`https://dashscope.aliyuncs.com/compatible-mode/v1`。
- 调用方式：`/chat/completions`。
- 测试期间没有修改本地 `config.toml`；阿里云百炼复测沿用当前 VoxType 简化 prompt 和相同合成样例，只替换请求模型名。
- 本记录不包含真实 API Key、真实识别正文、本地最近上下文、热词、场景偏好、日志或统计正文，也不复述实际 prompt 内容。

## 测试配置

- 2026-05-28 首次测试：使用当时 VoxType 默认大模型 System Prompt 和 User Prompt Template。
- 2026-05-30 修正复测：使用当前简化版 VoxType 默认大模型 System Prompt 和 User Prompt Template。
- 阿里云百炼复测：`enable_thinking = false`
- 超时：30 秒
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

## 2026-05-28 首次结果概览

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

## 2026-05-30 阿里云百炼修正复测

这轮复测使用当前简化 prompt 和相同 8 类合成样例。它仍然基于阿里云百炼 DashScope OpenAI 兼容 API，只替换模型名，没有修改本地配置。

| 模型 | 成功率 | 平均耗时 | 中位数 | 最慢一次 | 修正后评价 |
| --- | ---: | ---: | ---: | ---: | --- |
| `qwen3.6-flash-2026-04-16` | 8/8 | 518 ms | 528 ms | 569 ms | 最快，但技术路径样例会改错；提示词注入样例还出现语义删减 |
| `deepseek-v4-flash` | 8/8 | 809 ms | 750 ms | 1019 ms | 较快，但技术路径样例会改错 |
| `qwen3.7-max` | 8/8 | 989 ms | 835 ms | 1878 ms | 当前综合仍最均衡，但不能可靠还原代码路径 |
| `deepseek-v4-pro` | 8/8 | 970 ms | 927 ms | 1305 ms | 速度可用，但技术路径样例同样会改错，不再作为技术文本优先推荐 |
| `kimi-k2.6` | 8/8 | 1111 ms | 1080 ms | 1450 ms | 延迟比首次测试明显改善，但效果没有超过 `qwen3.7-max` |
| `qwen3.6-plus` | 8/8 | 1960 ms | 1784 ms | 4173 ms | 可用但有慢请求，技术路径样例会改错 |
| `glm-5.1` | 8/8 | 1984 ms | 1727 ms | 3803 ms | 稳定但偏保守，技术路径样例会改错 |
| `qwen3.6-max-preview` | 8/8 | 2034 ms | 2028 ms | 2938 ms | 慢，效果没有明显优势，技术路径样例会改错 |
| `qwen3.7-max-preview` | 0/8 | - | - | - | `enable_thinking=false` 下不可用，要求开启 thinking |

关键修正：

- 当前简化 prompt 下，多个模型都会把 ASR 形式的 `src tauri src llm post edit rs` 改成错误路径，例如把连字符、下划线或目录层级改错。不能再把 `deepseek-v4-pro` 作为“技术路径保真最好”的默认结论。
- `qwen3.7-max` 仍是百炼 API 下的综合优先选择，主要因为中文润色、问题不回答、隐私规则和提示词注入样例整体更均衡。
- `qwen3.6-flash-2026-04-16` 的速度优势更明显，但提示词注入样例中出现了删减原意的问题，只适合低风险、短句、低延迟优先的场景。
- `qwen3.7-max-preview` 与“关闭 thinking 以降低延迟”的实时润色目标冲突，不建议作为默认模型。

## DeepSeek 官方 API 补充复测

这一段不是阿里云百炼测试，而是为了修正同名模型在不同提供方下的误解。复测接口为 DeepSeek 官方 OpenAI 兼容 API，模型为 `deepseek-v4-pro`，使用当前简化 prompt。

3 条样例的思考关闭方式对比：

| 请求方式 | 成功率 | 平均耗时 | 最慢一次 | 现象 |
| --- | ---: | ---: | ---: | --- |
| `thinking = { type = "disabled" }` | 3/3 | 1355 ms | 1550 ms | 正确关闭思考，无 reasoning token，速度正常 |
| `enable_thinking = false` | 3/3 | 9808 ms | 14977 ms | 仍产生 reasoning token，明显变慢 |
| 省略思考字段 | 3/3 | 22058 ms | 23561 ms | reasoning token 更多，最慢 |

使用最佳方式 `thinking = { type = "disabled" }` 跑完整 8 条样例时，成功率为 8/8，平均 1252 ms，中位数 1188 ms，最慢 1584 ms。它没有回答问题或执行提示词注入文本，金融数字样例也能正确转换；但技术路径样例仍把 `src-tauri/src/llm_post_edit.rs` 类路径改错。

因此，官方 `deepseek-v4-pro` 可以作为低延迟可用模型，但必须使用 `thinking_strategy = "thinking_disabled"` 或让 VoxType 的 `thinking_strategy = "auto"` 测试后保存该策略；不建议把它作为代码路径、文件名和标识符文本的保真优先模型。

## 修正后的关键结论

`qwen3.7-max` 仍是当前百炼 API 下的日常默认优先选择。它不是最快，但整体质量和速度最均衡。

`qwen3.6-flash-2026-04-16` 仍是低延迟优先选择，但更适合普通短文本；遇到提示词样式文本、命令文本、代码路径或重要术语时风险更高。

`deepseek-v4-pro` 的旧推荐需要修正：无论百炼托管版本还是 DeepSeek 官方版本，在当前简化 prompt 下都没有可靠保住技术路径。技术路径、代码文件名、日志字段和英文标识符较多时，应优先依赖屏幕 OCR、热词/用户词典或人工检查，而不是仅靠切换到 `deepseek-v4-pro`。

`kimi-k2.6` 在 2026-05-30 复测中延迟已明显改善，但综合效果仍没有明显超过 `qwen3.7-max`。

## 推荐用法

日常默认优先考虑：

```toml
model = "qwen3.7-max"
```

低延迟优先时可考虑：

```toml
model = "qwen3.6-flash-2026-04-16"
```

如果使用 DeepSeek 官方 API：

```toml
model = "deepseek-v4-pro"
thinking_strategy = "thinking_disabled"
```

代码路径、文件名、日志字段和英文标识符较多时，不建议只靠换模型解决。更稳的做法是在录音时让屏幕 OCR 捕获上下文，或把常用路径、产品名、字段名加入热词/用户词典，并对最终文本做一次人工确认。

不建议作为 VoxType 实时润色默认选择：

```text
qwen3.7-max-preview
qwen3.6-max-preview
```

切换模型前，先确认该模型对当前 DashScope 账号和地域可用。切换后建议在 API 配置页重新点击大模型测试，确认权限、网络和延迟都符合预期。
