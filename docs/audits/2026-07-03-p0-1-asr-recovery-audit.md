# P0-1 ASR 异常恢复审计

日期：2026-07-03

## 目标

- 正式 ASR WebSocket 建连增加明确超时。
- 连接超时、连接失败、等待最终结果超时、连接提前关闭使用不同错误码。
- 异常后会话进入失败态，悬浮字幕短暂提示后隐藏。
- 下一次热键可重新开始，不停留在 `waiting_final_result`、`post_editing` 或 `pasting`。

## 变更范围

- `src-tauri/src/asr_ws.rs`：正式链路建连增加 20 秒超时；拆分 ASR 错误码；连接提前关闭不再混入 final timeout；补 ASR 单元测试。
- `src-tauri/src/session.rs`：新增内部异常重置能力，失败时让旧 generation 失效；补 session 单元测试。
- `src/lib/i18n/*`、`src/lib/utils/*`：补前端三语言错误说明和用户动作。
- README、英文 README、Wiki、ASR 守门文档和 CHANGELOG 同步用户可见行为。

## 主链路审计

- 空识别仍走 `EMPTY_TRANSCRIPT` 失败态，不触发 LLM、粘贴、最近上下文、自动热词历史或成功统计。
- 最终输出仍必须等待豆包最终包；没有最终包时失败，不使用实时字幕或中间包兜底成功。
- 没有改变 ASR 音频分片、停录尾音、OCR 等待、LLM 触发、剪贴板写入/恢复、热键默认值、托盘或统计策略。
- 日志和诊断只记录错误状态和长度类信息，不新增识别正文、OCR 正文、热词、prompt、最近上下文或密钥输出。

## 验证记录

- `npm run check`：通过。
- `cargo fmt`：通过。
- `cargo test asr_ws::tests`：通过，33 个测试。
- `cargo test session::tests`：通过，14 个测试。
- `cargo check`：通过。

## 手工回归建议

- 断网或阻断 `openspeech.bytedance.com` 后开始录音，确认约 20 秒内失败并显示连接类错误码。
- 录音停止后让服务端不返回最终包，确认显示 `ASR_FINAL_TIMEOUT` 且下一次快捷键可重新开始。
- 模拟 ASR 连接提前关闭，确认显示 `ASR_CONNECTION_CLOSED`，不会粘贴中间字幕。
