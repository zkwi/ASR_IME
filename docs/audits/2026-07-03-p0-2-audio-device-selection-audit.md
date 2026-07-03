# P0-2 麦克风设备选择审计

日期：2026-07-03

## 目标

- 不再只依赖易变的麦克风数字 index。
- 用户选过麦克风后，下次尽量按设备名称重新找到。
- 保存设备缺失时自动回退系统默认输入设备，并给非阻塞提示。

## 变更范围

- `src-tauri/src/config.rs`：新增 `audio.input_device_name`，保留旧 `audio.input_device` index 兼容旧配置。
- `src-tauri/src/audio.rs`：录音启动时优先按保存的设备名称匹配；缺失时回退默认输入设备；补纯函数测试覆盖 index 改变、设备缺失和默认设备可用。
- `src-tauri/src/session.rs`：麦克风回退时向前端发送 `audio-device-fallback` 事件。
- 前端类型、默认配置、设置页和三语言文案同步保存设备名称和展示回退通知。
- `config.example.toml`、README、英文 README、Wiki 和 CHANGELOG 同步说明。

## 主链路审计

- 只改变麦克风选择和回退提示，不改 ASR 请求参数、音频分片、停录尾音、LLM、剪贴板、热键、托盘或统计策略。
- 旧 `audio.input_device` 配置仍可读取；新配置优先使用 `audio.input_device_name`。
- 回退通知只包含设备名称，不包含识别正文、OCR、热词、prompt、最近上下文或密钥。

## 验证记录

- `npm run check`：通过。
- `cargo fmt`：通过。
- `cargo test audio::tests`：通过，25 个测试。
- `cargo test config::tests`：通过，19 个测试。
- `npm run ai:check`：通过，Rust 单元测试 203 个通过。

## 手工回归建议

- 选择一个非默认麦克风，保存后重启应用，确认仍显示并使用该麦克风。
- 改变设备枚举顺序或重连蓝牙耳机，确认同名设备仍被选中。
- 拔掉已保存麦克风后开始录音，确认自动回退默认麦克风并显示非阻塞提示。
