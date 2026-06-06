# 音频片段送达边界审计

日期：2026-06-06

## 目标

确认停录后所有本地已采集的语音片段都会进入豆包 ASR 识别链路，并且不会把中间识别结果当作最终文本使用。

## 豆包文档依据

- 大模型流式语音识别建议单包音频 100-200ms，发包间隔 100-200ms，过大或过小都会影响性能。
- 发送最后一包（负包）后，服务端会返回识别到的结果；双向流式优化版只在结果变化时返回新的数据包。
- `result_type` 默认为 `full`，返回累计结果；`single` 为增量结果。

对应实现仍保持 `result_type=full`，并且只在收到最终服务端包后产出最终文本。

## 当前发送链路

1. 麦克风回调读取原始输入格式。
2. `PcmSink` 将输入统一转换为豆包 ASR 使用的 16kHz、mono、PCM i16。
3. `SegmentedAudioBuffer` 按配置的分片时长切包。
4. 用户停止录音后，先停止采集流，再 flush `PcmSink`：
   - flush 重采样器里不足一个输出采样窗口的尾部样本；
   - flush 不足一个音频包的分片尾部。
5. ASR WebSocket 首包后先按正常分片节奏发送短头部静音，帮助服务端稳定识别开头。
6. ASR WebSocket 循环发送通道中所有已入队音频包；若启动阶段产生积压，也按 100-200ms 节奏发送，避免瞬间打包影响服务端处理。
7. 音频通道断开后，继续发送尾部静音包，再发送负包。
8. 负包后等待豆包最终包，并在短暂 settle 后选择最终文本。

## 已覆盖边界

- 48kHz 到 16kHz 下采样时，最后不足一个 16kHz 输出窗口的样本不会被丢弃。
- 停录时不足一个分片大小的 PCM 尾包会被发送。
- 非完整声道帧不会被 `chunks_exact` 静默丢弃。
- 首个真实音频包前会发送短静音，降低开头几个字被截断的概率。
- 麦克风启动成功后才显示录音悬浮字幕，避免在采集尚未就绪时提示用户开始说话。
- 真实音频、头部静音和尾部静音都按豆包建议的分片节奏发送。
- ASR 最终文本必须来自最终包；只有中间文本或 definitive 分句但没有最终包时会失败。
- 最终包比 definitive 分句包含更多尾字时，优先使用最终包文本。
- 负包前会补尾部静音，降低尾字被 VAD 截断的概率。

## 验证命令

```powershell
cargo test audio::tests
cargo test final_output
cargo test initial_audio_silence_chunks_prime_asr_before_first_real_audio
cargo test audio_send_pacer_keeps_documented_packet_interval_bounds
cargo test final_audio_silence_chunks
npm run ai:check
```

## 手工回归建议

- 说完最后一个字后立刻松开结束键，确认尾字保留。
- 按开始后立刻说短句，确认开头两个字保留。
- 长句连续输入，确认没有中间文本被粘贴。
- 很短的单字或两字输入，确认非空时能识别，空录音仍失败。
- 48kHz/44.1kHz/16kHz 麦克风设备各测试一次。
- 背景轻微噪音和短促键盘声场景下，确认静音停止不会提前截断。
