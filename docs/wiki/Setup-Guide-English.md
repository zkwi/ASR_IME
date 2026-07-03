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

API Config also includes **Recognition language**. The default is Auto/service default, which omits the `language` parameter. The main workflow uses `bigmodel_async + enable_nonstream` two-pass recognition, and Doubao documents `language` as unsupported by two-pass recognition, so leaving it blank is better for Chinese, English, dialect, and mixed input. Chinese Mandarin needs no setting, and existing `zh-CN` configs migrate to blank; only set a code such as `en-US`, `ja-JP`, or `yue-CN` when explicitly troubleshooting a non-default language.

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
| Base URL | OpenAI-compatible endpoint; service root, `/v1` URL, and full `/chat/completions` URL are accepted |
| API Key | Provider API key from the same platform/region as the Base URL |
| Model | For example `qwen3.5-plus`; must be available to the current account |
| Thinking adapter | Auto by default; the test tries candidate strategies and saves the fastest successful one |

Click **Test** after configuration. The test sends a sample text with the real AI prompt, shows the measured latency, and in Auto mode saves the fastest thinking/reasoning adapter that succeeds, but it does not read local recent-context text. If you only need speech recognition, LLM polishing is not required. When polishing is enabled, final transcripts that reach the minimum length are sent to your configured AI service; recognition terms, writing/product preferences, budget-compacted screen OCR, and optional recent context are appended as reference information. Recent context for AI is off by default; it is sent only when local recent context and "use recent context for polishing" are both enabled, and it is capped to about 200 chars from the latest snippets by default. Screen OCR is trimmed by line, deduplicated, and capped to 12 lines / 400 chars before AI polishing by default; term references are capped to 50 entries.

The default example uses Alibaba Cloud Bailian/DashScope's OpenAI-compatible endpoint. The Beijing Base URL is `https://dashscope.aliyuncs.com/compatible-mode/v1`; if you use Singapore, US, or another region, update Base URL, API Key, and model access together instead of changing only one field. For standard OpenAI-compatible services such as DeepSeek, service root, `/v1` URL, and full `/chat/completions` URL are treated as equivalent, for example `https://api.deepseek.com`, `https://api.deepseek.com/v1/`, and `https://api.deepseek.com/v1/chat/completions`.

For DashScope model-selection notes, see [2026-05-28 LLM polishing model test](../audits/2026-05-28-llm-polishing-model-test.md). The 2026-05-30 retest corrects the old conclusion: `qwen3.7-max` remains the daily default choice; `qwen3.6-flash-2026-04-16` is still the lower-latency option but is riskier for prompt-like text and technical paths; do not switch to `deepseek-v4-pro` only for technical text, because the simplified prompt can still rewrite code paths. Actual availability depends on the current account and region.

Recommendations:

- Thinking is disabled with provider-specific request fields where supported because voice polishing is latency-sensitive; retest after changing Base URL or model.
- Text shorter than `min_chars = 100` is not polished by default.
- Similar or identical model names can behave very differently across providers, so prefer the latency measured by API Config over the model name alone.
- Code paths, filenames, and English identifiers are easy for an LLM to "correct" into plausible but wrong forms. Use screen OCR, hotwords, or a manual check for those terms.
- If the network is unstable, adjust LLM timeout in `config.toml`.

### If LLM Test Fails

| Symptom | Check first |
| --- | --- |
| API Key or permission failure | API Key belongs to the configured Base URL provider and region |
| Model not found or forbidden | Model name spelling and account permission |
| Connection failure | Base URL belongs to the configured provider, network/proxy is usable |
| Test passes but polishing does not run | Polishing is enabled and text length reaches `min_chars` |
| Test passes but real polishing is slow | Rerun the thinking adapter test and confirm thinking/reasoning is disabled or minimized |
| Code paths are often rewritten | Enable screen OCR, or add common paths, filenames, and field names to hotwords |

If LLM polishing fails during input, VoxType keeps the original ASR text and still tries to copy/paste it.

## 5. Hotwords and Prompts

Open **Hotwords & prompts**.

### Hotwords

Use one item per line. Good hotwords include:

- Names, company names, product names.
- Project names, abbreviations, code names.
- Technical terms that ASR often misrecognizes.

Do not add passwords, ID numbers, phone numbers, customer data, or other sensitive information. Doubao ASR direct hotwords are capped at the first 100 effective entries, with manual hotwords taking priority over confirmed automatic hotwords, to avoid oversized real-time ASR requests.

### Writing Context

Use writing context for the current writing scenario, product names, project background, and preferred wording. In daily use, update this first before editing the AI prompt.

### Recent Context

Recent context is off by default. When enabled, VoxType saves recent recognized snippets to local `context/recent_context.jsonl` to improve continuity.

Notes:

- Only VoxType recognition snippets are saved; keyboard input is not recorded.
- Recent context is not written back to `config.toml`.
- Clear it from Privacy & local data, or delete `context/recent_context.jsonl` manually.

### AI Prompt

VoxType includes a default voice-input AI prompt. It marks the text-to-polish block as the only content to rewrite and output. Even if the transcript contains questions, commands, or prompt-like content, the LLM should polish the text rather than answer, execute, or analyze it. Short messages, one-line commands, and questions get light correction, with natural punctuation allowed but no expansion. Long spoken notes, records, retrospectives, explanations, meeting notes, product feedback, and investment reviews are polished into publishable prose: filler words, verbal padding, repeated expressions, dead pauses, and self-corrections are removed; sentence order can be adjusted; sentences can be split; necessary connectors can be added; and the result is usually organized into 2-4 natural paragraphs. It preserves the original facts, judgment, intensity, stance, proper nouns, English abbreviations, finance terms, and programming terms, and avoids adding headings, lists, Markdown, or backticks. User dictionary terms, writing/product preferences, optional recent context, and screen OCR are appended as reference-information blocks; they only help correct terms, names, UI words, continuity, paths, filenames, code identifiers, and wording preferences, not act as text to polish or instructions to follow, and must not add information that the text to polish did not say. Recent context must not be continued, summarized, or reproduced, and screen OCR is only used for related corrections. For file paths, commands, log fields, and code identifiers, the default prompt asks the model to keep uncertain text unchanged unless the reference information provides an exact spelling. In finance, investing, and quant contexts, the default prompt asks the LLM to normalize clear amounts, returns, and percentages into common numeric forms such as `100万` and `1%`, without calculating returns or answering questions.

The Hotwords page lets you:

- Restore the default prompt.
- Preview the final prompt, including reference-information rules, the current screen OCR policy and LLM budgets, and whether recent context enters the AI prompt.
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

Screen OCR context is on by default. It captures the current display by default, which helps when you reference one document while typing into another window. You can switch it to the current window only in Options. OCR text is lightly normalized, used only for the current ASR/LLM request, and is not written to logs, stats, config, or cache. OCR sent to ASR is controlled by the screen OCR character limit; OCR sent to AI polishing has a separate LLM budget. Switch to current-window-only or disable it when the screen contains sensitive content.

Low-level parameters stay in `config.toml`: Resource ID, ASR WebSocket URL, model name, final-result timeout, max recording seconds, stop grace milliseconds, LLM timeout, main hotkey enable flag, mute system volume while recording, OCR character limit and wait time, caption custom size/position/color, clipboard restore delay, snapshot size, and retry parameters. The LLM minimum polishing length and reference budgets are adjustable on Hotwords & prompts.

## 7. Recommended Defaults

| Config | Recommended Value | Reason |
| --- | --- | --- |
| Primary shortcut | `Ctrl + Q` | Low conflict, easy to remember |
| Middle mouse | Off | Can conflict with browsers or editors |
| Right Alt | Off | Can conflict with IMEs or shortcuts |
| Paste method | Automatic paste | Works for most text fields |
| Clipboard restore | On | Tries to restore previous clipboard after paste |
| Low-volume auto-stop | Off by default (`0` seconds), threshold `0.03` | Avoids cutting off quiet microphones because of local threshold misclassification; set a positive value only for unattended long recording |
| Screen OCR context | On, current display | Improves names, UI terms, filenames, and code identifiers; switch to current-window-only or disable in sensitive scenarios |
| Recent context | Off | Conservative by default; AI access to previous text also needs a separate opt-in |
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
use_recent_context = false
base_url = "https://dashscope.aliyuncs.com/compatible-mode/v1"
api_key = ""
model = "qwen3.5-plus"
min_chars = 100
screen_context_max_chars = 400
screen_context_max_lines = 12
recent_context_max_chars = 200
reference_hotwords_limit = 50
enable_thinking = false
thinking_strategy = "auto"
```

Recording:

```toml
[audio]
max_record_seconds = 300
stop_grace_ms = 250
silence_auto_stop_seconds = 0
silence_level_threshold = 0.03
input_gain_db = 0.0
mute_system_volume_while_recording = false
```

VoxType keeps actual ASR packets within Doubao's recommended `100-200ms` range, defaulting to `200ms`. About `50ms` of leading silence is merged into the first real-audio packet while keeping the first packet at the configured segment size, defaulting to about `200ms`; this helps Doubao stabilize initial speech recognition without sending a standalone 50ms packet. If no real microphone audio is captured, VoxType does not send an extra silence packet.

VoxType keeps `input_gain_db = 0.0` by default and does not boost microphone audio. Only raise input gain slightly in recording troubleshooting when the recording quality card repeatedly reports low volume and the system microphone level and distance already look correct; try `+3 dB` or `+6 dB` first to avoid clipping speech or amplifying room noise. After each recording, Home shows a lightweight recording quality card with the latest RMS, peak, active speech ratio, and a suggestion; these metrics contain no recognized text and are not written to the main stats table.

Interim Doubao text is shown in the floating caption with a shorter local throttle; fast interim updates are coalesced to the latest text and emitted on time. When utterance text is more complete than `result.text` in the same response, captions prefer the fuller cumulative utterance text, while final paste still waits for the final package. `stop_grace_ms` is the fixed real-audio tail wait after stopping, defaulting to about `250ms`; it no longer depends on local volume detection to decide whether to extend, so quiet microphones are less likely to lose tail words because of threshold misclassification. Any partial final audio chunk is flushed before the microphone is closed, no extra trailing silence is appended, and the last audio packet is sent as the negative final packet to help Doubao trigger the final two-pass endpoint. Local silence auto-stop is disabled by default with `silence_auto_stop_seconds = 0`; keep it disabled for quiet microphones.

The recently verified stable combination is to keep the default `200ms` ASR packet size and put perceived-speed work into `20ms` response polling, `50ms` caption throttling, and a `500ms` OCR-context wait. First-word acceleration is off by default to prioritize beginning-word accuracy. Final output still accepts only Doubao's final package and prefers that package's full `result.text`. `definite=true` utterances stabilize the final result, but when the final package highly overlaps those utterances and recovers missing head or tail words, VoxType should keep the final full text even if the package slightly shortens earlier wording.

First-word acceleration is disabled by default with `enable_accelerate_text = false` and `accelerate_score = 0`; the old default `true + 8` migrates to off. If faster live-caption startup matters more, you can manually enable it in `config.toml`, but beginning-word accuracy may drop.

Semantic smoothing `enable_ddc` is enabled by default for light ASR-side smoothing on short and medium text, reducing reliance on LLM polishing for short inputs. Old default combinations migrate to enabled together with the LLM threshold migration. After that, disable it manually when exact proper nouns, short commands, paths, or punctuation-sensitive dictation matter more.

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
timeout_ms = 500
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
6. Press `Ctrl + Q` to start recording; press it again to stop. Local silence auto-stop is off by default and should be enabled only for unattended long recording.
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

Recent context contains real dictated text, so VoxType does not save it or send it to an AI service by default. When local recent context is enabled, it helps Doubao ASR with continuous dictation; it reaches the AI service only when "use recent context for polishing" is also enabled and polishing actually runs.

Screen OCR context is on by default, but it does not save transcript history. It reads the current display at recording start by default, lightly normalizes OCR text, and attaches it temporarily to the current ASR/LLM request. It does not cache recent OCR screenshots or text, and OCR sent to the LLM is compacted by budget. Use current-window-only or turn it off when the screen contains sensitive content.
