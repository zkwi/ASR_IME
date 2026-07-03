# 配置旧默认迁移跟进审计

日期：2026-07-03

## 背景

`min_chars` 修复后继续排查同类问题：按字段当前数值猜测旧默认值的迁移无法区分“旧版本默认配置”和“用户刚手动保存的合法值”，会造成设置被重置。

## 发现

以下配置存在同类风险：

- `auto_hotwords.max_history_chars = 10000` 会被改为 `5000`。
- `audio.silence_auto_stop_seconds = 10` 或 `30` 会被改为 `0`。
- `audio.silence_level_threshold = 0.04` 会被改为 `0.03`。
- `audio.stop_grace_ms = 100`、`150`、`200` 或 `800` 会被改为 `250`。
- `request.enable_accelerate_text = true` 且 `request.accelerate_score = 8` 会被改为关闭首字加速。
- 扩大审计最近约 15 个提交时，还发现配置保存命令虽然过滤了“未改动隐藏高级字段”的校验错误，但实际写入仍调用完整校验路径，导致普通设置保存可能继续失败。

## 修复

- 移除以上按数值猜旧默认的迁移。
- 新配置或缺失字段继续使用当前默认值。
- 已保存的显式配置值一律保留。
- 保留 `result_type = "single"` 到 `"full"` 的兼容规范化，因为 ASR 请求实际已强制发送 `full`。
- 保留 `request.language = "zh-CN"` 到空值的兼容规范化，因为二遍识别不会发送 `zh-CN`，且前端只展示自动/服务默认和非默认语种。
- 保存配置时，若剩余错误全部来自未改动的隐藏高级字段，改走已确认的内部写入路径，避免第二次完整校验再次拦截；可见字段或已改动字段错误仍会阻止保存。

## 验证

- 已新增失败回归测试 `load_config_preserves_configured_values_that_match_old_defaults`，确认修复后通过。
- 已新增失败回归测试 `unchanged_hidden_config_validation_errors_use_unchecked_save_mode`，确认隐藏字段保存路径修复后通过。

## 手工回归建议

- 在配置中分别设置上述旧默认相同数值，启动或保存配置后确认数值不被改写。
- API 配置页保留“自动 / 服务默认”的识别语言展示，非默认语种仍可保存。
