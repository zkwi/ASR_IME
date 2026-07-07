# ASR 无反馈自动停止审计

日期：2026-07-07

## 结论

本次改动将旧的“本地音量静音自动停止”废弃，替换为 session 层的“ASR 无有效反馈自动停止”。默认值为 `30` 秒，配置字段为 `[asr].no_feedback_auto_stop_seconds`，`0` 表示关闭。

## 主链路影响

- 自动停止由 `session.rs` 统一执行，触发后复用 `stop_generation_with_grace`，仍会保留停录尾音、关闭麦克风、flush 尾部 PCM，并等待 ASR provider 的最终完成事件。
- 豆包只在收到非空实时文本、非空最终候选、有效 definite 分句更新或带文本最终包时上报活动。
- 阿里云只在收到非空 partial/stable 文本，或 `task-finished` 前已有最终分句文本时上报活动；`task-started` 和空文本不算有效反馈。
- 空识别、最终包门禁、LLM 润色、剪贴板、统计和日志脱敏规则不变。

## 配置与兼容

- 新字段：`asr.no_feedback_auto_stop_seconds = 30`。
- 合法范围：`0..=300`。
- 旧字段 `audio.silence_auto_stop_seconds` 和 `audio.silence_level_threshold` 不再进入配置模型；旧配置文件加载时会被忽略，重新保存后不会写回。

## 验证重点

- 默认配置应通过校验，并使用 30 秒无反馈超时。
- `0` 和 `300` 应通过校验，`301` 应失败。
- 手动停止、最大录音时长停止、ASR 无反馈停止都应保持 generation 门禁，不能让旧 worker 覆盖新会话。
- 豆包最终输出仍必须等待最终包，阿里云最终输出仍必须等待 `task-finished`。
