# VoxType Wiki - Windows Voice Typing and Speech-to-Text

本页面是 GitHub Wiki `Home` 的仓库内草稿镜像，用于避免线上 Wiki 与仓库文档长期漂移。

VoxType is a lightweight Windows desktop voice typing, dictation, and speech-to-text tool. Put the cursor in any input box, press the global shortcut, speak, and VoxType will transcribe your voice with Doubao streaming ASR, optionally polish the result with an OpenAI-compatible LLM, copy it to the clipboard, paste it into the active field, and restore the previous clipboard when possible.

声写 VoxType 是一个 Windows 桌面语音输入、语音转文字和听写工具。把光标放到任意输入框后，按全局快捷键开始说话，VoxType 会完成录音、豆包流式 ASR、可选大模型润色、写入剪贴板、自动粘贴和剪贴板恢复。

## Use Cases / 适合场景

- Windows voice typing and speech-to-text in chat apps, browsers, editors, forms, and office tools.
- 中文语音输入、英文听写、多语言语音转文字，以及需要实时字幕的桌面输入场景。
- Doubao ASR transcription with optional LLM polishing for cleaner long-form dictation.
- A local-first open-source workflow with conservative privacy defaults.

## Interface Preview / 界面预览

Home keeps the current input state, shortcut triggers, latest result notice, and input performance in one screen. 首页顶部集中展示语音输入状态和启动方式；识别完成后会提示已复制并尝试粘贴，并提供临时复制或查看本次识别文本的入口。

<img src="https://raw.githubusercontent.com/zkwi/VoxType/main/screenshots/ScreenShot_2026-05-09_130744_793.png" alt="VoxType 中文首页：语音输入状态、启动方式和输入表现" width="820">

API Config shows setup health before the credential forms. API 配置页先展示 ASR 密钥、麦克风、粘贴方式、触发方式和隐私设置状态，再提供豆包 ASR 与可选大模型测试入口。截图中的密钥已脱敏。

<img src="https://raw.githubusercontent.com/zkwi/VoxType/main/screenshots/ScreenShot_2026-05-09_130827_317.png" alt="VoxType English API Config and setup health check" width="820">

## 简体中文

- [用户配置指南](Setup-Guide)
- [功能特性与使用优化](Feature-Guide)
- [常见问题与排障](Troubleshooting)

## English

- [User Configuration Guide](Setup-Guide-English)
- [Features and Usage Optimization](Feature-Guide-English)
- [Troubleshooting](Troubleshooting-English)

## Recommended Reading Order

1. Configure Doubao ASR first.
2. Add hotwords and optional prompt preferences.
3. Adjust daily preferences such as shortcut, paste mode, microphone, captions, startup, and tray behavior.
4. Open `config.toml` only when troubleshooting low-level ASR, LLM timeout, caption geometry, or clipboard timing parameters.

## Current Defaults Worth Knowing

- Main trigger: `Ctrl + Q`.
- Right Alt and middle mouse triggers: off by default.
- Recent context and automatic hotword candidates: off by default.
- Screen OCR context: on by default, current foreground window only, no persisted OCR text.
- Local silence fallback: continuous low volume stops recording after 30 seconds by default, with a `0.03` threshold.
- Doubao server endpointing example value: `end_window_size = 800`.
- Update prompts provide an "Update now" action when a new installer is available.

## Privacy Basics

- `config.toml`, logs, local context files, and usage stats are local files and should not be committed.
- Usage statistics record duration, character count, speed, and time estimates, not recognized text.
- Recent context and automatic hotword history are disabled by default.
- Screen OCR context is temporary and not written to logs, stats, config, or cache.
- The Privacy & local data page shows config, logs, storage/upload boundaries, and can clear recent context, automatic hotword history, and usage stats.
- Diagnostic reports and logs should not include real API keys, recognized text, hotwords, prompts, recent context, screen OCR text, automatic hotword history, or Windows username paths.

## Search-Friendly FAQ

### Is VoxType a Windows dictation app?

Yes. VoxType is a Windows desktop dictation app that turns microphone speech into text and pastes it into the active input field.

### Does VoxType require Doubao ASR?

Yes for the main speech-to-text workflow. Doubao ASR App Key and Access Key are required before recording, recognition, and paste actions are unlocked.

### Is LLM polishing required?

No. LLM polishing is optional. VoxType can run as a pure ASR voice input tool, and only calls an OpenAI-compatible LLM when polishing is enabled and configured.
