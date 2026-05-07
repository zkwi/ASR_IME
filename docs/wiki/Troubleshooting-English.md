# VoxType Troubleshooting

This page is the repository draft mirror for the GitHub Wiki `Troubleshooting-English` page, so the Wiki and repository docs do not drift apart.

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

### How to Confirm the Fix

- The VoxType main window shows the Home page.
- WebView2-related processes start together with VoxType in Task Manager.
- Microsoft Edge or other WebView2 apps can render pages normally.

### What to Include When Reporting

- Windows version.
- VoxType version.
- Whether the machine has used system debloating tools, Edge/WebView2 uninstallers, browser component blocking policies, or security software blocks.
- Whether the WebView2 overwrite install succeeded or showed an error.
- Whether Microsoft Edge opens normally.

Do not send real API keys, hotwords, prompts, recognized text, unredacted logs, or screenshots that expose Windows username paths.

## Microphone Does Not Record

Check Windows permission first:

```text
Windows Settings -> Privacy & security -> Microphone -> Let desktop apps access your microphone
```

Then select the correct input device in VoxType Options. If you use a Bluetooth headset, confirm that Windows is not still using a hands-free call input device.

## Shortcut Does Not Respond

- The default trigger is `Ctrl + Q`.
- Right Alt and middle mouse are disabled by default and must be enabled manually.
- If another app owns `Ctrl + Q`, choose a different global shortcut.
- Target apps running as administrator may block simulated input from a normal-permission app. Try running VoxType with matching permissions.

## Recognition Works but Text Is Not Pasted

Press `Ctrl + V` manually first. If the text appears, recognition and clipboard writing succeeded, and the target app probably blocked simulated paste.

In Options, try:

- Switch between automatic paste and clipboard-only output.
- Adjust paste delay.
- Test another target input field.

## Update Fails

If in-app update fails, download the latest installer manually from GitHub Releases:

<https://github.com/zkwi/VoxType/releases>

You usually do not need to uninstall the old version first. If the installer says files are in use, exit VoxType from the tray and try again.
