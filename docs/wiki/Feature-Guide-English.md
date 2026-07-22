# Windows Voice Typing Features and Usage Optimization

This page is the repository draft mirror for the GitHub Wiki `Feature-Guide-English` page, so the Wiki and repository docs do not drift apart.

This page explains VoxType's Windows voice typing, speech-to-text, selectable ASR service, automatic paste, clipboard restore, and optional LLM polishing features. It focuses on when each feature is useful and how to tune it for speed, latency, and recognition quality.

简体中文版本：[功能特性与使用优化](Feature-Guide)

## 1. Core Workflow

```text
Trigger recording → Capture microphone audio → Selected ASR service → Optional LLM polishing → Write clipboard → Auto paste → Restore clipboard → Stats and logs
```

Main workflow guarantees:

- Empty recognition becomes a failure and does not trigger polishing, paste, or successful statistics.
- The UI only shows "polishing" when LLM polishing is enabled, polishing length reaches `min_chars`, and Base URL, API Key, and model are complete.
- Floating captions show real-time text, elapsed state, and errors only.
- Usage statistics do not store recognized text.

## 2. Home

The Home page is for daily use:

- Check current input state.
- Start or stop recording.
- See main and backup trigger states.
- See recent 24-hour, 7-day, speed, and saved time estimates.
- After a successful input, temporarily view, copy, or immediately clear the latest recognized text in the current window. It is not written into usage stats, logs, or diagnostic reports, and is cleared when the window is hidden, the app exits, or a new recording starts.

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
- ASR no-feedback auto-stop: `30s` by default, `0` disables it
- Microphone input gain: `0 dB`

VoxType normalizes the actual microphone input to `16000Hz`, mono, 16-bit PCM before sending it to the selected ASR service. `sample_rate` and `channels` are capture preferences, not the final wire format.

Tips:

- Select a fixed input device if you have multiple microphones.
- Low volume, long distance, noisy rooms, and disabled Windows microphone permission can cause empty recognition.
- If the microphone is quiet but clear, check the system microphone level and distance first; if the recording quality card still repeatedly reports low volume, raise input gain slightly in recording troubleshooting to `+3 dB` or `+6 dB`.
- A microphone input-stream error fails the current session immediately, so incomplete audio is not polished, pasted, or counted as success.
- ASR no-feedback auto-stop defaults to 30 seconds. If the provider returns no effective text feedback for that window, VoxType stops through the normal grace flow and does not depend on local volume thresholds.
- System-volume mute while recording is off by default; enable it only if echo affects recognition.

## 5. ASR Service

VoxType uses Doubao `bigmodel_async` WebSocket by default, and Alibaba Cloud FunASR can be selected from API Config. Both providers share the same recording, caption, final text, LLM, clipboard, and stats workflow.

Doubao mode keeps two-pass recognition enabled by default. Live captions are feedback only, while pasted output waits for Doubao's final package, prefers final `definite=true` utterances, and can use a highly overlapping final full text to recover missing head or tail words. Alibaba Cloud FunASR mode waits for `task-finished` and only uses final sentence text as output. If the connection closes early or the final wait times out, VoxType fails the session instead of pasting interim text. In Doubao mode, the main workflow forces two-pass recognition, utterance output, and `full` cumulative result delivery. First-word acceleration is disabled by default to prioritize beginning-word accuracy, while DDC semantic smoothing is enabled by default for light ASR-side smoothing on short and medium text.

While a session is waiting for its final result, pressing any enabled input trigger interrupts the old session and immediately starts a new recording. A late result from the interrupted session cannot continue to polishing, paste, or stats.

Quality and latency factors:

| Factor | Recommendation |
| --- | --- |
| Audio segment | Keep the internal default 200ms |
| Microphone input gain | Default 0 dB; quiet but clear mics can try a small +3 dB or +6 dB boost |
| Server endpointing | `end_window_size` defaults to 800ms; existing manual config is preserved |
| First-word return | `enable_accelerate_text` defaults to off with `accelerate_score = 0`; enable it manually only when faster live-caption startup matters more |
| Semantic smoothing | `enable_ddc` is on by default; disable it manually when exact proper nouns, short commands, paths, or punctuation-sensitive dictation matter more |
| Result type | The main workflow forces `full` so the final package contains the complete cumulative text; interim captions are feedback only |
| ASR no-feedback fallback | Defaults to 30 seconds; set `no_feedback_auto_stop_seconds = 0` to disable |
| Final result timeout | Default 15s; adjust in `config.toml` only for network/service issues |
| Hotwords | Important for proper nouns and product names |
| Recent context | Useful for continuous writing, but disabled by default for privacy; when enabled, it is sent to the selected ASR service |
| Screen OCR context | On by default, current display by default, useful for UI terms, filenames, and code identifiers |

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
- Text below `min_chars` is not polished. CJK characters count individually, while English and numbers count by contiguous word-like segments.
- Thinking is disabled with provider-specific adapters where supported, and the LLM test saves the fastest successful adapter.
- When polishing is enabled and the text reaches the minimum length, the final transcript is sent to your configured AI service; recognition terms, writing/product preferences, budget-compacted screen OCR, and optional recent context are appended as reference information.
- Recent context for AI is off by default. It is sent only when local recent context and "use recent context for polishing" are both enabled, and it is capped to about 200 chars from the latest snippets by default.
- Before AI polishing, screen OCR is trimmed by line, deduplicated, and capped to 12 lines / 400 chars by default; term references are capped to 50 entries.

Tips:

- Short messages often do not need polishing.
- Documentation, meeting notes, and requirement drafts benefit more from polishing.
- If polishing is slow, first confirm the automatic adapter test after auto-save has finished, then rerun the thinking adapter test manually if needed, disable thinking, raise `min_chars`, or choose a faster model. Raising timeout can reduce failures, but it does not make the model faster.
- If the text contains code paths, filenames, log fields, or English identifiers, let screen OCR and hotwords provide the exact spelling, and still check the final text manually.

## 8. Hotwords, Scene Notes, and Prompts

Use hotwords for terms that ASR often misrecognizes:

```text
VoxType
Tauri
Doubao ASR
Project code names
Product names
```

Terms or context sent to the selected ASR service are capped according to provider capability. Doubao ASR direct hotwords are capped at the first 100 effective entries, with manual hotwords taking priority over confirmed automatic hotwords, to avoid oversized real-time ASR requests. Keep frequent proper nouns and remove stale or rarely used terms.

Use scene notes for long-term style and context:

```text
I often dictate product requirements, code review notes, and project plans.
Keep output concise and do not expand.
Preserve English technical terms and project names.
```

In daily use, start with hotwords and scene notes. Open Advanced prompt settings only when you need a fixed rewriting style:

- User Prompt template and minimum polishing length are in Advanced prompt settings on Hotwords & prompts.
- LLM reference budgets and System Prompt stay in `config.toml`.
- Final prompt preview shows how recognition terms, writing/product preferences, optional recent context, and screen OCR are appended as budgeted reference information.
- The default prompt separates short and long text: short messages, one-line commands, and questions get light correction, with natural punctuation allowed but no expansion; long spoken notes, records, retrospectives, explanations, meeting notes, product feedback, and investment reviews are polished into publishable prose, usually as 2-4 natural paragraphs.
- The default prompt preserves proper nouns, English abbreviations, finance terms, and programming terms.
- The default prompt preserves the original facts, judgment, intensity, and stance, and avoids adding headings, lists, Markdown, or backticks.
- The default prompt labels user dictionary terms, writing preferences, optional recent context, and screen OCR as reference information, not text to polish or instructions to follow, and prevents reference-only details from being added when the text to polish did not say them.
- The default prompt marks the text-to-polish block as the only output source. Recent context must not be continued, reproduced, or summarized, and screen OCR is only used for related path, filename, code identifier, and UI-term corrections.
- File paths, commands, log fields, and code identifiers are kept unchanged when uncertain, unless reference information provides an exact spelling.
- In finance, investing, and quant contexts, the default prompt asks the LLM to normalize clear amounts, returns, and percentages into common numeric forms such as `100万` and `1%`, without calculating or answering questions.

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

Screen OCR context reads text from the current display or current window when recording starts. It helps ASR and the optional LLM understand UI terms, filenames, names, and code identifiers.

Privacy and stability:

- On by default, with the current display as the capture range. You can switch to current-window-only in Options.
- OCR text is used only for the current request and is not written to logs, stats, config, or cache.
- The Options-page OCR test preview can be cleared immediately and is cleared automatically when the window is hidden, the app exits, or a new recording starts.
- When included in LLM polishing, OCR is trimmed, deduplicated, budget-capped, and appended as a separate reference-information block that is explicitly marked as not text to polish or user instructions.
- Failure or timeout is skipped automatically and does not block recording, ASR, polishing, clipboard, or paste.
- Switch to current-window-only or disable it in Options when the screen contains sensitive content; Privacy & local data can jump back to that setting through Manage settings.

## 11. Privacy & Local Data

Privacy & local data answers four questions in one place: what VoxType stores, where it lives, whether it is uploaded, and whether it can be cleared. The page shows counts, switch state, logical paths, and upload boundaries only. It does not show transcripts, OCR text, hotword history text, prompts, or keys.

The page separates data into three groups:

- Base local files: the config file and local logs. The config file can contain API keys and basic settings, but recent context text is kept out of `config.toml`. Logs and diagnostic reports are redacted by default and should not contain transcripts, hotwords, prompts, or raw keys.
- Local clearable data: recent context, automatic hotword history, and usage stats. Recent context and automatic hotword history contain voice-input text history and are off by default. Usage stats contain non-content metrics such as character count, duration, and speed. Recent context is sent to the selected ASR service after local recent context is enabled; it reaches the AI service only after a separate AI opt-in.
- Runtime and third-party service data: ASR audio, screen OCR, LLM polishing text, and temporary clipboard snapshots. These are normally not written to disk, but ASR audio, OCR context, optional recent context, and LLM polishing text can be sent with the current request depending on enabled features.

Clear actions remove VoxType local files only. They do not mean third-party ASR/LLM providers have deleted data they already received; retention depends on the provider configured by the user.

## 12. Floating Captions

Captions are designed for recording feedback:

- Real-time text.
- Elapsed or processing state.
- Errors.
- Captions collapse formatting line breaks and repeated whitespace from interim ASR text. Short text stays on one line; long or width-wrapped text uses up to two visible lines with the largest font size that does not clip. The runtime window is at least `52px` high, including for legacy low-height settings.
- An anomalous one-to-four-character interim drop in the same recording session does not replace an already complete caption; substantial revisions still update normally.

Captions do not show paste-state noise, internal paths, or debug stacks.

Use presets first. Fine-tune width, height, colors, and bottom margin in `config.toml` only if captions block your content. If long captions should more consistently show two lines, increase `height` instead of shortening ASR audio segments.

## 13. Statistics

Stats record:

- Session count
- Voice duration
- Character count
- Average speed
- Saved time estimate

Saved time is estimated as manual typing time for the same character count minus actual voice duration. The default manual typing baseline is about 50 Chinese characters per minute.

Stats do not store recognized text.

## 14. Tray, Startup, and Updates

Tray:

- Closing the main window hides it to the tray by default.
- Single-click the tray icon to open the main window.
- Tray menu can open the main window, open config, open logs, report an issue, check updates, restart the app, and exit.

Startup:

- Can be enabled from Options.

Updates:

- Manual update check is available in Options and the tray menu.
- Startup auto-check is enabled by default.
- When a new version is available, notices and the update panel provide an "Update now" action.
- Updates download the NSIS installer from GitHub Releases, exit the current app to release files, and try to open the new version after installation.

## 15. Latency Optimization

| Goal | Tune first |
| --- | --- |
| Faster real-time captions | Keep the internal 200ms audio segments and ensure network stability |
| Faster final output | Disable LLM polishing or increase `min_chars` |
| Faster polishing | Rerun the thinking adapter test, keep thinking disabled, and choose a faster model |
| More reliable paste | Keep clipboard restore enabled and increase restore delay if needed |
| Fewer accidental triggers | Keep right Alt and middle mouse disabled |
| Better proper nouns | Maintain hotwords, scene notes, and screen OCR context |
