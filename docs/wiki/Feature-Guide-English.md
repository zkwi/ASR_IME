# Windows Voice Typing Features and Usage Optimization

This page is the repository draft mirror for the GitHub Wiki `Feature-Guide-English` page, so the Wiki and repository docs do not drift apart.

This page explains VoxType's Windows voice typing, speech-to-text, Doubao streaming ASR, automatic paste, clipboard restore, and optional LLM polishing features. It focuses on when each feature is useful and how to tune it for speed, latency, and recognition quality.

简体中文版本：[功能特性与使用优化](Feature-Guide)

## 1. Core Workflow

```text
Trigger recording → Capture microphone audio → Doubao streaming ASR → Optional LLM polishing → Write clipboard → Auto paste → Restore clipboard → Stats and logs
```

Main workflow guarantees:

- Empty recognition becomes a failure and does not trigger polishing, paste, or successful statistics.
- The UI only shows "polishing" when LLM polishing is enabled, text reaches `min_chars`, and Base URL, API Key, and model are complete.
- Floating captions show real-time text, elapsed state, and errors only.
- Usage statistics do not store recognized text.

## 2. Home

The Home page is for daily use:

- Check current input state.
- Start or stop recording.
- See main and backup trigger states.
- See recent 24-hour, 7-day, speed, and saved time estimates.
- After a successful input, temporarily view and copy the latest recognized text in the current window. It is not written into usage stats, logs, or diagnostic reports, and is cleared when the window closes or a new recording starts.

## 3. Global Triggers

Default trigger: `Ctrl + Q`.

| Trigger | Default | Recommendation |
| --- | --- | --- |
| Main shortcut | On | Keep it on |
| Middle mouse | Off | Enable only after checking browser/editor conflicts |
| Right Alt | Off | Enable only after checking IME/system shortcut conflicts |

Avoid enabling multiple easy-to-misfire triggers unless you really need them.

## 4. Microphone Capture

VoxType uses Rust `cpal` to capture PCM audio.

Defaults. These are low-level parameters, so most users do not need to change them in the settings UI; edit `config.toml` only when troubleshooting:

- Sample rate: `16000`
- Channels: `1`
- Segment size: `200ms`
- Max recording duration: `300s`
- Local low-volume auto-stop: `30s`, threshold `0.03`

Tips:

- Select a fixed input device if you have multiple microphones.
- Low volume, long distance, noisy rooms, and disabled Windows microphone permission can cause empty recognition.
- If a recording contains no useful speech, VoxType uses the local silence fallback to stop recording instead of waiting for the maximum duration.
- System-volume mute while recording is off by default; enable it only if echo affects recognition.

## 5. Doubao Streaming ASR

VoxType uses Doubao `bigmodel_async` WebSocket by default.

It supports real-time partial text and final recognition. Low-level ASR request fields remain supported in `config.toml`, but ordinary users do not need to edit them.

Quality and latency factors:

| Factor | Recommendation |
| --- | --- |
| Audio segment | Keep the internal default 200ms |
| Server endpointing | `end_window_size` defaults to 800ms; existing manual config is preserved |
| Local silence fallback | Continuous low volume follows the manual-stop flow after 30 seconds by default |
| Final result timeout | Default 15s; adjust in `config.toml` only for network/service issues |
| Hotwords | Important for proper nouns and product names |
| Recent context | Useful for continuous writing, but disabled by default for privacy |
| Screen OCR context | On by default, current foreground window only, useful for UI terms, filenames, and code identifiers |

## 6. Auto Paste and Clipboard Restore

After recognition, VoxType:

1. Writes the final text to the clipboard.
2. Sends `Ctrl+V` or `Shift+Insert`.
3. Tries to restore the previous clipboard.

Options exposes:

- `Ctrl+V`: default and suitable for most text fields.
- `Shift+Insert`: useful for apps that intercept `Ctrl+V`.
- Clipboard only: useful when you do not want VoxType to send paste keys.
- Restore clipboard after paste: on by default.

Low-level clipboard restore delay, snapshot size, and retry parameters stay in `config.toml`.

If paste fails but text was copied, press `Ctrl + V` manually.

## 7. LLM Polishing

LLM polishing is useful for:

- Cleaning dictated long sentences.
- Removing filler words and repetition.
- Structuring long text into paragraphs or bullet-like lines.
- Preserving proper nouns with hotwords and scene notes.

Defaults:

- ASR only unless LLM polishing is enabled.
- Short text below `min_chars` is not polished.
- Thinking is disabled to reduce latency.

Tips:

- Short messages often do not need polishing.
- Documentation, meeting notes, and requirement drafts benefit more from polishing.
- If polishing is slow, disable thinking, raise `min_chars`, or choose a faster model.

## 8. Hotwords, Scene Notes, and Prompts

Use hotwords for terms that ASR often misrecognizes:

```text
VoxType
Tauri
Doubao ASR
Project code names
Product names
```

Use scene notes for long-term style and context:

```text
I often dictate product requirements, code review notes, and project plans.
Keep output concise and do not expand.
Preserve English technical terms and project names.
```

Prompt editing:

- User Prompt template is available from the default page.
- Minimum polishing length is available on Hotwords & prompts.
- System Prompt stays in `config.toml`.
- Final prompt preview is available.

## 9. Automatic Hotword Candidates

Automatic hotword candidates extract possible terms from local voice input history.

Privacy behavior:

- Off by default.
- Stores only VoxType final voice input text.
- Does not record keyboard input.
- Does not read clipboard history.
- Local history can be cleared from Hotwords & prompts or Privacy & local data.
- Calls the configured LLM only when you manually generate candidates.
- Candidates must be confirmed before joining hotwords.

Enable it when you frequently dictate recurring business terms, product names, or people names. Keep it off when your dictated content is highly sensitive.

## 10. Screen OCR Context

Screen OCR context reads text from the current foreground window when recording starts. It helps ASR and the optional LLM understand UI terms, filenames, names, and code identifiers.

Privacy and stability:

- On by default, but captures only the current foreground window, not the full screen.
- OCR text is used only for the current request and is not written to logs, stats, config, or cache.
- Failure or timeout is skipped automatically and does not block recording, ASR, polishing, clipboard, or paste.
- Disable it in Options or Privacy & local data when the current foreground window contains sensitive content.

## 11. Floating Captions

Captions are designed for recording feedback:

- Real-time text.
- Elapsed or processing state.
- Errors.

Captions do not show paste-state noise, internal paths, or debug stacks.

Use presets first. Fine-tune width, height, colors, and bottom margin in `config.toml` only if captions block your content.

## 12. Statistics

Stats record:

- Session count
- Voice duration
- Character count
- Average speed
- Saved time estimate

Saved time is estimated as manual typing time for the same character count minus actual voice duration. The default manual typing baseline is about 50 Chinese characters per minute.

Stats do not store recognized text.

## 13. Tray, Startup, and Updates

Tray:

- Closing the main window hides it to the tray by default.
- Tray menu can open the main window, open config, open logs, report an issue, check updates, restart the app, and exit.

Startup:

- Can be enabled from Options.

Updates:

- Manual update check is available in Options and the tray menu.
- Startup auto-check is enabled by default.
- When a new version is available, notices and the update panel provide an "Update now" action.
- Updates download the NSIS installer from GitHub Releases, exit the current app to release files, and try to open the new version after installation.

## 14. Latency Optimization

| Goal | Tune first |
| --- | --- |
| Faster real-time captions | Keep the internal 200ms audio segments and ensure network stability |
| Faster final output | Disable LLM polishing or increase `min_chars` |
| Faster polishing | Disable thinking and choose a faster model |
| More reliable paste | Keep clipboard restore enabled and increase restore delay if needed |
| Fewer accidental triggers | Keep right Alt and middle mouse disabled |
| Better proper nouns | Maintain hotwords, scene notes, and screen OCR context |
