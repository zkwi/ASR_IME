# 音频片段送达边界审计

日期：2026-06-06

## 目标

确认停录后所有本地已采集的语音片段都会进入豆包 ASR 识别链路，并且不会把中间识别结果当作最终文本使用。

## 豆包文档依据

- 大模型流式语音识别建议单包音频 100-200ms，发包间隔 100-200ms，双向流式 200ms 分包最优；过大或过小都会影响性能。
- 发送最后一包（负包）后，服务端会返回识别到的结果；双向流式优化版只在结果变化时返回新的数据包。
- `enable_nonstream` 开启二遍识别后，实时文本用于快速反馈，`definite=true` 的二遍分句用于提升最终准确率。
- `show_utterances` 需要开启，才能获得分句与 `definite` 标记。
- `result_type` 默认为 `full`，返回累计结果；`single` 为增量结果。
- `enable_ddc` 文档默认关闭；它偏语义顺滑，可能改写专有词、短命令或标点敏感口述。
- `enable_accelerate_text` 可提升首字返回速度，但会降低首字准确率；当前按用户要求默认开启，并使用中等 `accelerate_score=8`。
- 直传热词/上下文应控制规模，避免实时 ASR 请求过大影响性能。

对应实现强制保持 `enable_nonstream=true`、`show_utterances=true`、`result_type=full`，默认关闭 `enable_ddc`，并且只在收到最终服务端包后产出最终文本。

## 当前发送链路

1. 麦克风回调读取原始输入格式。
2. `PcmSink` 将输入统一转换为豆包 ASR 使用的 16kHz、mono、PCM i16。
3. `SegmentedAudioBuffer` 按配置的分片时长切包。
4. 用户停止录音后，先停止采集流，再 flush `PcmSink`：
   - flush 重采样器里不足一个输出采样窗口的尾部样本；
   - flush 不足一个音频包的分片尾部。
5. ASR WebSocket 将约 50ms 头部静音并入第一包真实音频，第一包总时长仍按配置分片发送，默认约 200ms，帮助服务端稳定识别开头且避免独立 50ms 小包。
6. ASR WebSocket 循环发送通道中所有已入队音频包；若启动阶段产生积压，也按每个包的实际音频时长安排下一次发送，并限制在 100-200ms，避免瞬间打包或短包长等影响服务端处理；等待下一包发送期间仍继续读取豆包响应，不阻塞实时反馈。
7. 音频通道断开后，不再追加尾部静音；将 flush 后的最后一个音频包直接作为负包发送。
8. 负包后等待豆包最终包；此时不再沿用发包节流，避免 1ms 轮询造成额外抖动。
9. 收到最终包后最多短暂 settle 约 300ms，再选择最终文本。

## 已覆盖边界

- 48kHz 到 16kHz 下采样时，最后不足一个 16kHz 输出窗口的样本不会被丢弃。
- 停录时不足一个分片大小的 PCM 尾包会被发送。
- 非完整声道帧不会被 `chunks_exact` 静默丢弃。
- 约 50ms 头部静音会并入第一包真实音频，降低开头几个字被截断的概率，同时不产生独立 50ms 小包；没有真实音频时不会补发静音包。
- ASR 实际采集分片会限制在 100-200ms；默认 200ms 保留豆包双向流式推荐值。
- 录音悬浮字幕会立即显示启动状态；麦克风启动成功后才切换为正在听你说话，避免在采集尚未就绪时提示用户开始说话。
- 只有队列开头补约 50ms 静音；中间真实音频分片和停录后的尾部分片保持原样，不额外补静音。
- 第一包总时长仍按配置分片发送，默认 200ms；不足一片的尾包按实际音频时长安排发送，不再按默认 200ms 人为拉长。
- 音频发送节奏等待不阻塞 WebSocket 响应读取；豆包返回的实时累计文本优先显示到悬浮字幕，字幕刷新靠更短本地节流和更频繁响应轮询提速，不通过缩短默认 200ms 音频包提速。
- 首字返回加速默认开启以改善字幕跟手感；最终输出仍只使用最终包，不会因为开启加速而接受中间文本。
- `enable_ddc` 新默认关闭；已有配置里的显式取值保留，避免覆盖用户选择。
- ASR 直传热词限制为前 100 个有效条目，手动热词优先于自动确认热词。
- 屏幕 OCR 上下文默认只等待 300ms，超时正常跳过，避免 ASR 建连前长时间等待可选上下文。
- 负包后的最终结果等待使用正常响应轮询节奏，减少停录后的本地 CPU 抖动。
- ASR 最终文本必须来自最终包；只有中间文本或 definitive 分句但没有最终包时会失败。
- 最终包比 definitive 分句包含更多尾字时，优先使用最终包文本。
- 最后一个音频包直接使用负序列发送，降低尾字被 VAD 截断的概率，同时避免额外空负包拉长停录后的等待。

## 验证命令

```powershell
cargo test audio::tests
cargo test final_output
cargo test initial_audio_padding_keeps_first_packet_at_configured_segment_size
cargo test initial_audio_padding_does_not_repeat_between_real_packets
cargo test audio_send_pacer_uses_actual_packet_duration_with_documented_bounds
cargo test final_wait_uses_default_response_poll_timeout_after_audio_finished
cargo test config::tests
cargo test asr::tests
npm run ai:check
```

## 手工回归建议

- 说完最后一个字后立刻松开结束键，确认尾字保留。
- 按开始后立刻说短句，确认开头两个字保留。
- 长句连续输入，确认没有中间文本被粘贴。
- 很短的单字或两字输入，确认非空时能识别，空录音仍失败。
- 48kHz/44.1kHz/16kHz 麦克风设备各测试一次。
- 背景轻微噪音和短促键盘声场景下，确认静音停止不会提前截断。
