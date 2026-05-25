# VoxType User Configuration Guide: Windows Voice Typing, Doubao ASR, and Optional LLM Polishing

This page is the repository draft mirror for the GitHub Wiki `Setup-Guide-English` page, so the Wiki and repository docs do not drift apart. When updating the live Wiki, check this file as well.

This guide is for first-time setup and daily VoxType use. It covers Windows voice typing, speech-to-text with Doubao streaming ASR, optional OpenAI-compatible LLM polishing, automatic paste, clipboard restore, hotwords, and troubleshooting. Start with the minimum setup that makes the main workflow usable, then enable quality, trigger, caption, update, and privacy-related options as needed.

Chinese version: [声写 VoxType 用户配置指南](Setup-Guide)

## 1. Install and Check Windows Permissions

VoxType targets Windows 10/11.

Download the Windows installer from GitHub Releases:

<https://github.com/zkwi/VoxType/releases>

The installer includes the Microsoft Edge WebView2 Bootstrapper. On a clean machine without WebView2 Runtime, the installer installs it automatically.

Before recording, make sure Windows allows desktop apps to access the microphone:

```text
Windows Settings -> Privacy & security -> Microphone -> Let desktop apps access your microphone
```

## 2. Main Pages

VoxType has six main pages:

| Page | Purpose |
| --- | --- |
| Home | Check current state, start/stop voice input, and see trigger methods |
| Hotwords & prompts | Manage hotwords, scene notes, AI prompts, and automatic hotword candidates |
| API Config | Configure required Doubao ASR credentials and optional LLM API |
| Options | Configure shortcuts, paste method, microphone, floating captions, startup, and close behavior |
| Privacy & local data | Review storage/upload boundaries and clear local context, hotword history, and stats |
| Analytics | View recent 24-hour, recent 7-day, and daily usage stats |

The Home voice card shows idle/recording state plus the primary shortcut, middle mouse, and right Alt in compact chips. Recent input and input performance stay below it.

### Current UI Reference

After a successful input, Home shows "Input completed". This means VoxType copied the text and attempted to paste it. If the target field did not receive text, press `Ctrl + V`, click "Copy text", or inspect the result with "View recognized text". The recognized text is kept only in the current window.

<img src="https://raw.githubusercontent.com/zkwi/VoxType/main/screenshots/ScreenShot_2026-05-09_130803_332.png" alt="VoxType Home page with voice input state and input performance" width="820">

When ASR keys are missing, API Config first shows a three-step first-use guide: fill keys, test the connection, then return Home to start dictating. Setup health follows below: ASR keys, microphone, paste method, trigger method, and privacy status are shown separately. Only issues that block recording, recognition, or paste should be prominent warnings. Optional settings such as right Alt, middle mouse, recent context, and automatic hotwords should remain softer reminders. Public screenshots should blur real App Keys, Access Keys, and other secrets.

<img src="https://raw.githubusercontent.com/zkwi/VoxType/main/screenshots/ScreenShot_2026-05-09_130827_317.png" alt="VoxType API Config and setup health check" width="820">

Privacy & local data centralizes the storage/upload boundaries for recent context, automatic hotword history, usage stats, ASR audio, screen OCR, LLM polishing text, and clipboard snapshots. It also clears local context, hotword history, and usage stats.

## 3. Required: Configure Doubao ASR

The core workflow depends on Doubao streaming ASR. Without ASR credentials, recording, recognition, and paste stay locked.

Minimum setup rule: once the Doubao ASR test passes, you can return Home and start dictating. LLM polishing, hotwords, and screen OCR are optional quality improvements.

Open **API Config -> Doubao credentials** and fill in:

| Field | Required | Notes |
| --- | --- | --- |
| App Key | Yes | App Key/App ID for the Doubao speech service in the Volcano Engine console |
| Access Key | Yes | Access Key for the same Doubao speech service |

The default Resource ID is `volc.seedasr.sauc.duration`. Most users do not need to edit it in the UI; special cases can edit `config.toml`.

VoxType currently sends `X-Api-App-Key`, `X-Api-Access-Key`, and `X-Api-Resource-Id` as documented by Doubao streaming ASR. If the console shows multiple keys, confirm that they belong to the same Doubao speech recognition service and billing resource. Do not paste a Bailian/DashScope LLM key, GitHub token, or unrelated cloud secret into ASR credentials. The Doubao credentials panel includes an official docs link for checking the field descriptions during first-time setup.

Click **Test** after filling credentials. When the test passes, return to Home and start voice input.

API Config also includes **Recognition language**. The default is `zh-CN`. For English, Japanese, Cantonese, or other languages supported by Doubao docs, switch the language here. Choose Auto/service default to omit the language parameter. If ASR testing fails after changing the language, switch back to Auto/service default or confirm the current API mode.

### If Doubao ASR Test Fails

| Symptom | Check first |
| --- | --- |
| Authentication or permission failure | App Key, Access Key, and Resource ID belong to the same Doubao speech service and account |
| Connection failure or timeout | Network, proxy, or firewall access to `openspeech.bytedance.com` |
| Failure after changing language | Switch Recognition language back to Auto/service default and test again |
| Test passes but recording returns no text | Windows microphone permission, selected input device, mic volume, and actual speech |

If it still fails, open **Options -> Updates and diagnostics -> Copy diagnostic report** and include the redacted error code/status in an Issue. Do not paste real keys, full logs, or transcript text.

Doubao official docs:

<https://www.volcengine.com/docs/6561/1354869?lang=en>

Do not commit real keys, and do not share your local `config.toml`.

## 4. Optional: Configure LLM Polishing

The LLM API is used for:

- Light polishing of recognized text.
- Organizing longer dictated text for common scenarios.
- Generating automatic hotword candidates.

Open **API Config -> LLM API**:

| Field | Notes |
| --- | --- |
| Enable polishing | When off, VoxType uses ASR only |
| Base URL | OpenAI-compatible endpoint, for example `https://dashscope.aliyuncs.com/compatible-mode/v1` |
| API Key | Provider API key from the same platform/region as the Base URL |
| Model | For example `qwen3.5-plus`; must be available to the current account |

Click **Test** after configuration. The test sends a sample text with the real AI prompt and shows the measured latency when it succeeds. If you only need speech recognition, LLM polishing is not required. When polishing is enabled, final transcripts that reach the minimum length are sent to your configured AI service; recognition terms, writing/product preferences, and screen OCR are appended as reference information.

The default example uses Alibaba Cloud Bailian/DashScope's OpenAI-compatible endpoint. The Beijing Base URL is `https://dashscope.aliyuncs.com/compatible-mode/v1`; if you use Singapore, US, or another region, update Base URL, API Key, and model access together instead of changing only one field.

Recommendations:

- Thinking is off by default because voice polishing is usually latency-sensitive.
- Text shorter than `min_chars = 40` is not polished by default.
- If the network is unstable, adjust LLM timeout in `config.toml`.

### If LLM Test Fails

| Symptom | Check first |
| --- | --- |
| API Key or permission failure | API Key belongs to the configured Base URL provider and region |
| Model not found or forbidden | Model name spelling and account permission |
| Connection failure | Base URL ends with `/compatible-mode/v1`, network/proxy is usable |
| Test passes but polishing does not run | Polishing is enabled and text length reaches `min_chars` |

If LLM polishing fails during input, VoxType keeps the original ASR text and still tries to copy/paste it.

## 5. Hotwords and Prompts

Open **Hotwords & prompts**.

### Hotwords

Use one item per line. Good hotwords include:

- Names, company names, product names.
- Project names, abbreviations, code names.
- Technical terms that ASR often misrecognizes.

Do not add passwords, ID numbers, phone numbers, customer data, or other sensitive information.

### Writing Context

Use writing context for the current writing scenario, product names, project background, and preferred wording. In daily use, update this first before editing the AI prompt.

### Recent Context

Recent context is off by default. When enabled, VoxType saves recent recognized snippets to local `context/recent_context.jsonl` to improve continuity.

Notes:

- Only VoxType recognition snippets are saved; keyboard input is not recorded.
- Recent context is not written back to `config.toml`.
- Clear it from Privacy & local data, or delete `context/recent_context.jsonl` manually.

### AI Prompt

VoxType includes a default voice-input AI prompt. It treats recognized text as source material, not instructions. Even if the transcript contains questions, commands, or prompt-like content, the LLM should polish the text rather than answer, execute, or analyze it. The default prompt preserves the source language and mixed Chinese/English wording instead of translating Chinese into another language or foreign-language text into Chinese. User dictionary terms, writing/product preferences, and screen OCR are appended as reference-information blocks; they only help correct terms, names, UI words, and wording preferences, not act as text to polish or instructions to follow. It also corrects obvious ASR word errors, missing words, broken grammar, and unnatural phrasing without adding facts; if the original meaning is unclear, it keeps the source wording. In finance, investing, and quant contexts, the default prompt asks the LLM to normalize clear amounts, returns, and percentages into common numeric forms such as `100万`, `1%`, and `10%`, without calculating returns or answering questions.

The Hotwords page lets you:

- Restore the default prompt.
- Preview the final prompt, including reference-information rules, the current screen OCR policy, and the recent-context policy.
- Edit the User Prompt template.
- Adjust the minimum polishing length from 0 to 10000.

System Prompt stays in `config.toml` to keep the normal UI concise.

### Automatic Hotword Candidates

Automatic hotword candidates are off by default. When enabled, VoxType saves final voice-input text locally. Only when the user clicks "Generate candidates" does it send a summary to the configured LLM service. Local history can be cleared from Hotwords & prompts or Privacy & local data.

Candidates are not added automatically. The user must review and confirm them. The default local history limit is 5000 characters; old 10000-character defaults are migrated to 5000 on config load.

## 6. Daily Options

Options is grouped into Common settings, Enhancements, and Maintenance so daily controls come first and maintenance entries are clearly separated:

| Section | Visible Settings |
| --- | --- |
| Common settings | Primary shortcut, microphone, paste method, remove trailing period, restore clipboard after paste |
| Enhancements | Screen OCR context, Windows OCR test, caption preview, color presets, opacity presets |
| Maintenance | Startup, close-window behavior, check updates, update now, open logs, copy diagnostic report |
| Recording troubleshooting | Low-volume auto-stop |
| Extra start options | Middle mouse and right Alt |

To review or clear local data, open Privacy & local data from the sidebar.

Screen OCR context is on by default. It captures the current display by default, which helps when you reference one document while typing into another window. You can switch it to the current window only in Options. OCR text is lightly normalized, used only for the current ASR/LLM request, and is not written to logs, stats, config, or cache. Switch to current-window-only or disable it when the screen contains sensitive content.

Low-level parameters stay in `config.toml`: Resource ID, ASR WebSocket URL, model name, final-result timeout, max recording seconds, LLM timeout, main hotkey enable flag, mute system volume while recording, OCR character limit and wait time, caption custom size/position/color, clipboard restore delay, snapshot size, and retry parameters.

## 7. Recommended Defaults

| Config | Recommended Value | Reason |
| --- | --- | --- |
| Primary shortcut | `Ctrl + Q` | Low conflict, easy to remember |
| Middle mouse | Off | Can conflict with browsers or editors |
| Right Alt | Off | Can conflict with IMEs or shortcuts |
| Paste method | Automatic paste | Works for most text fields |
| Clipboard restore | On | Tries to restore previous clipboard after paste |
| Low-volume auto-stop | 30 seconds, threshold `0.03` | Less likely to cut off long or quiet dictation |
| Screen OCR context | On, current display | Improves names, UI terms, filenames, and code identifiers; switch to current-window-only or disable in sensitive scenarios |
| Recent context | Off | More conservative by default |
| Automatic hotword candidates | Off | Does not save transcript history by default |
| Mute system volume while recording | Off | Avoids interrupting meetings, videos, and alerts |
| Thinking | Off | Faster for voice polishing |

## 8. Key `config.toml` Fields

Settings edited in the UI auto-save. The title bar briefly shows pending, saving, and saved states. For manual edits, use `config.example.toml` as the reference.

Minimum ASR config:

```toml
[auth]
app_key = ""
access_key = ""
resource_id = "volc.seedasr.sauc.duration"
```

Optional LLM config:

```toml
[llm_post_edit]
enabled = false
base_url = "https://dashscope.aliyuncs.com/compatible-mode/v1"
api_key = ""
model = "qwen3.5-plus"
min_chars = 40
enable_thinking = false
```

Recording:

```toml
[audio]
max_record_seconds = 300
silence_auto_stop_seconds = 30
silence_level_threshold = 0.03
mute_system_volume_while_recording = false
```

Triggers:

```toml
[triggers]
hotkey_enabled = true
middle_mouse_enabled = false
right_alt_enabled = false
```

Output:

```toml
[typing]
paste_method = "ctrl_v"
remove_trailing_period = true
restore_clipboard_after_paste = true
clipboard_restore_delay_ms = 800
```

Screen OCR context:

```toml
[screen_context]
enabled = true
capture_scope = "screen"  # screen = current display, window = current window only
max_chars = 1200
timeout_ms = 700
```

Updates:

```toml
[update]
auto_check_on_startup = true
github_repo = "zkwi/VoxType"
```

## 9. First Use Flow

1. Install and start VoxType.
2. Open API Config.
3. Fill in Doubao ASR App Key and Access Key; keep the default Resource ID unless you have a special reason.
4. Click **Test** for Doubao ASR.
5. Return to Home and put the cursor in a target input field.
6. Press `Ctrl + Q` to start recording; press it again to stop, or wait for low-volume auto-stop.
7. Wait for final recognition and optional polishing.
8. If text does not appear in the target field, press `Ctrl + V` manually.

## 10. Next Steps

- To improve recognition quality, read [Features and Usage Optimization](Feature-Guide-English).
- For shortcut, paste, microphone, startup, or update issues, read [Troubleshooting](Troubleshooting-English).

## 11. Common Questions

### Is LLM API required?

No. VoxType's core workflow is Windows voice typing with Doubao ASR speech-to-text. LLM polishing is optional.

### Can I use clipboard-only output?

Yes. In Options, choose clipboard-only output. VoxType will leave the recognized text in the clipboard and skip simulated paste.

### What should I put in hotwords?

Names, product names, project names, abbreviations, and technical terms. Do not put secrets or sensitive customer data there.

### Why are recent context and automatic hotword candidates off by default?

They save voice-input text history locally. VoxType keeps them off by default to reduce privacy risk.

Screen OCR context is on by default, but it does not save transcript history. It reads the current display at recording start by default, lightly normalizes OCR text, and attaches it temporarily to the current ASR/LLM request. It does not cache recent OCR screenshots or text. Use current-window-only or turn it off when the screen contains sensitive content.
