# VoxType Troubleshooting

This page is the repository draft mirror for the GitHub Wiki `Troubleshooting-English` page, so the Wiki and repository docs do not drift apart.

简体中文版本：[常见问题与排障](Troubleshooting)

## Quick Checklist

When something fails, check in this order:

1. API Config setup health has no red blocking item.
2. Doubao ASR App Key and Access Key are filled in.
3. Windows allows desktop apps to access the microphone.
4. Home shows at least one enabled trigger.
5. The cursor is in an editable text field.
6. The target app does not block paste shortcuts.
7. Options can open logs or copy a redacted diagnostic report.

## Blank Startup Window or Stuck Startup Page

### Symptoms

- VoxType opens to a blank main window.
- VoxType stays on the startup page for a long time.
- Reinstalling VoxType does not restore the main UI.
- Other Windows desktop apps that use WebView2 may also fail.

### Cause

VoxType is a Tauri desktop app. Its main window is rendered by Microsoft Edge WebView2 Runtime. If the system WebView2 Runtime is broken, missing, outdated, or blocked by policy, the app frontend cannot load, so VoxType cannot show its normal troubleshooting UI.

For that reason, VoxType should not silently download, elevate, or repair system components from inside the app. The installer handles missing Runtime installation. If the Runtime is already present but broken, repair it manually with the official Microsoft installer.

### Recommended Fix

1. Close VoxType.
2. Open the official Microsoft WebView2 download page: <https://developer.microsoft.com/microsoft-edge/webview2/>
3. Download **Evergreen Standalone Installer**. For most Windows 10/11 PCs, choose `x64`.
4. Save the installer locally, for example `C:\Temp\MicrosoftEdgeWebView2RuntimeInstallerX64.exe`.
5. Right-click the installer and choose "Run as administrator" to run a repair/overwrite install.
6. Start VoxType again after the installer finishes.
7. If the window is still blank, restart Windows and try VoxType again.

You can also run this from an administrator PowerShell:

```powershell
Start-Process -Wait -Verb RunAs "C:\Temp\MicrosoftEdgeWebView2RuntimeInstallerX64.exe" -ArgumentList "/silent", "/install"
```

## `Ctrl + Q` Does Nothing

Possible causes:

- ASR credentials are missing, so the main workflow is locked.
- The main shortcut is disabled.
- The shortcut is occupied by another app.
- VoxType is starting, stopping, waiting for the final result, polishing, or outputting text.
- The target app runs as administrator and blocks simulated input from a normal-permission app.

Fix:

1. Open API Config and confirm required ASR fields are filled in.
2. Open Options and confirm the main shortcut is `Ctrl + Q` and at least one trigger is enabled.
3. Try another shortcut.
4. If the target app runs as administrator, try starting VoxType with matching permissions.

## No Text Was Recognized

Possible causes:

- Microphone permission is disabled.
- Wrong input device.
- Microphone volume is too low or too far away.
- The recording contains no useful speech.
- Network or ASR service issue.

Fix:

1. Allow desktop microphone access in Windows Settings.
2. Select the correct input device in Options.
3. Test Doubao ASR from API Config.
4. Try again in a quieter environment.

Notes:

- Empty recognition becomes a failure. It does not run polishing, paste, or successful statistics.
- Continuous low volume follows the manual-stop flow after 30 seconds by default, so a server endpointing miss does not record until the maximum duration.
- If you need long pauses, adjust or disable the local silence fallback in `config.toml`.

## Doubao ASR Test Fails

First confirm that App Key, Access Key, and Resource ID all belong to the same Doubao speech recognition service. VoxType currently sends `X-Api-App-Key`, `X-Api-Access-Key`, and `X-Api-Resource-Id`; do not paste an LLM API key, GitHub token, or unrelated cloud secret into ASR credentials.

Common fixes:

1. Authentication or permission failure: copy App Key and Access Key again, confirm Doubao streaming ASR is enabled, and confirm Resource ID matches the billing resource.
2. Connection failure or timeout: check proxy, firewall, and network access to `openspeech.bytedance.com`.
3. Failure after changing recognition language: switch back to Auto/service default and test again.
4. Test passes but recording returns no text: check Windows microphone permission, input device, and mic volume.

When asking publicly, include only redacted diagnostic error codes and statuses. Do not paste real keys, full logs, or transcript text.

## Recognition Works but Text Is Not Pasted

Press `Ctrl + V` manually first. If the text appears, recognition and clipboard writing succeeded, and the target app probably blocked simulated paste.

When Home shows "Input completed", VoxType has copied the recognized text and attempted to paste it. Use "Copy text" to write it to the clipboard again, or "View recognized text" to inspect the latest result. This text is kept only in the current window and is cleared when the window closes or the next recording starts.

Fix:

1. Test in Notepad first.
2. If text was copied, press `Ctrl + V` manually.
3. Try `Shift + Insert` or clipboard-only output in Options.
4. If the target app reads the clipboard slowly, increase clipboard restore delay in `config.toml`.

## Previous Clipboard Was Not Fully Restored

VoxType tries to restore common clipboard formats. Images, bitmap handles, file handles, large private formats, or very large clipboard content may not be fully backed up.

Suggestions:

- Plain text and common rich text are usually more stable.
- Large clipboard content may hit the snapshot size limit.
- If restore is partial, VoxType should keep the recognized text available and show a warning.
- If you often handle large images, tables, or file lists, temporarily disable clipboard restore or use clipboard-only output.

## LLM Polishing Does Not Run

Possible causes:

- LLM polishing is disabled.
- Base URL, API Key, or model is incomplete.
- Text length is below `min_chars`.
- LLM connection test fails.

Fix:

1. Enable LLM polishing in API Config.
2. Fill in Base URL, API Key, and model.
3. Run the LLM test.
4. Confirm Base URL, API Key, and model come from the same LLM platform/region.
5. Leave it off if short text does not need polishing.

Notes:

- The default DashScope / Alibaba Cloud Bailian Beijing Base URL is `https://dashscope.aliyuncs.com/compatible-mode/v1`.
- For standard OpenAI-compatible services such as DeepSeek, service root, `/v1` URL, and full `/chat/completions` URL are treated as equivalent, for example `https://api.deepseek.com`, `https://api.deepseek.com/v1/`, and `https://api.deepseek.com/v1/chat/completions`.
- Common Base URL mistakes are using the wrong provider address, or using a key from another platform/region.
- The model name must be available to the account; missing model access fails the test.
- If LLM polishing fails during input, VoxType keeps using the original ASR text.

## LLM Polishing Is Slow

Try:

- Rerun the LLM test in API Config so VoxType saves the fastest thinking adapter, and keep thinking disabled.
- Increase `min_chars` so short text skips polishing.
- Use a faster model.
- Increase timeout only for slow networks or models; timeout does not make polishing faster.

## Hotwords Do Not Help Much

Hotwords are context, not a forced replacement table.

Tips:

- Use one term per line.
- Use real spelling.
- Do not put long paragraphs into hotwords.
- Proper nouns, names, product names, and abbreviations work best.
- For style preferences, use scene notes instead.

## Automatic Hotword Candidates Are Empty

Possible causes:

- Automatic hotwords are disabled.
- Local history is empty.
- LLM API is not configured.
- There are no high-quality candidate terms.

Fix:

1. Enable automatic hotword candidates on Hotwords & prompts.
2. Use voice input a few times to accumulate local history.
3. Configure and test LLM API.
4. Generate candidates manually.

## Screen OCR Text Has Extra Spaces

Since 0.1.62, VoxType merges extra spaces between adjacent CJK characters before sending screen OCR context to Doubao ASR and the optional LLM. For example, `屏 幕 OCR 上 下 文` is normalized to `屏幕 OCR 上下文`. English acronyms, shortcuts, paths, and number spacing are kept as much as possible.

If the test preview is still obviously poor, confirm that the Windows Chinese OCR language capability is available, and keep the reference text clear and unobstructed. If the screen contains sensitive information, switch to current-window-only or disable Screen OCR context in Options; recording, ASR, and paste still work without it.

## Update Fails

If in-app update fails, download the latest installer manually from GitHub Releases:

<https://github.com/zkwi/VoxType/releases>

You usually do not need to uninstall the old version first. If the installer says files are in use, exit VoxType from the tray and try again.

## Logs and Diagnostic Report

Options and the tray menu can open logs. Options can also copy a redacted diagnostic report.

When reporting an issue, include:

- VoxType version.
- Windows version.
- Steps to reproduce.
- Whether the issue reproduces in Notepad.
- Redacted diagnostic report.

Do not send real API keys, hotwords, prompts, recognized text, screen OCR text, automatic hotword history, unredacted logs, or screenshots that expose Windows username paths.
