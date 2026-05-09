<script lang="ts">
  import type {
    AppConfig,
    AudioDeviceInfo,
    HotkeyCaptureState,
    ScreenContextTestResult,
    SoftConfigNoticeKey,
    UpdateStatus,
  } from "$lib/types/app";
  import type { CopyKey } from "$lib/i18n";
  import { ClipboardCopy, Download, FileText, Keyboard, ScanText, ShieldCheck } from "lucide-svelte";

  type Translate = (key: CopyKey, values?: Record<string, string>) => string;
  type OverlayColorPreset = { label: CopyKey; background: string; text: string };

  type Props = {
    config: AppConfig;
    t: Translate;
    hotkeyCaptureState: HotkeyCaptureState;
    hotkeyValidationMessage: string;
    overlayColorPresets: OverlayColorPreset[];
    overlayOpacityPresets: readonly number[];
    audioDevices: AudioDeviceInfo[];
    updateStatus: UpdateStatus | null;
    checkingUpdate: boolean;
    installingUpdate: boolean;
    openingLog: boolean;
    copyingDiagnosticReport: boolean;
    testingScreenContext: boolean;
    screenContextTestResult: ScreenContextTestResult | null;
    fieldError: (field: string) => string;
    formatHotkey: (value: string) => string;
    overlayBackgroundRgb: () => string;
    overlayOpacity: () => number;
    overlayTextColor: () => string;
    overlayPresetActive: (background: string, text: string) => boolean;
    overlayOpacityPresetActive: (opacity: number) => boolean;
    overlayOpacityLabel: (opacity: number) => string;
    updatePanelTitle: () => string;
    updatePanelDescription: () => string;
    updateMetaText: () => string;
    onHotkeyKeydown: (event: KeyboardEvent) => void;
    onBeginHotkeyCapture: () => void;
    onOptionEnabledNotice: (key: SoftConfigNoticeKey, enabled: boolean) => void;
    onApplyOverlayPreset: (background: string, text: string) => void;
    onApplyOverlayOpacity: (opacity: number) => void;
    onSetInputDevice: (value: string) => void;
    onCheckUpdate: (manual: boolean) => void;
    onDownloadLatestUpdate: () => void;
    onOpenLog: () => void;
    onCopyDiagnosticReport: () => void;
    onTestScreenContext: () => void;
    onOpenPrivacyPage: () => void;
  };

  let {
    config = $bindable<AppConfig>(),
    t,
    hotkeyCaptureState,
    hotkeyValidationMessage,
    overlayColorPresets,
    overlayOpacityPresets,
    audioDevices,
    updateStatus,
    checkingUpdate,
    installingUpdate,
    openingLog,
    copyingDiagnosticReport,
    testingScreenContext,
    screenContextTestResult,
    fieldError,
    formatHotkey,
    overlayBackgroundRgb,
    overlayOpacity,
    overlayTextColor,
    overlayPresetActive,
    overlayOpacityPresetActive,
    overlayOpacityLabel,
    updatePanelTitle,
    updatePanelDescription,
    updateMetaText,
    onHotkeyKeydown,
    onBeginHotkeyCapture,
    onOptionEnabledNotice,
    onApplyOverlayPreset,
    onApplyOverlayOpacity,
    onSetInputDevice,
    onCheckUpdate,
    onDownloadLatestUpdate,
    onOpenLog,
    onCopyDiagnosticReport,
    onTestScreenContext,
    onOpenPrivacyPage,
  }: Props = $props();

</script>

<section class="settings-stack">
  <section class="settings-group">
    <div class="settings-group-heading">
      <h3>{t("optionsPageTitle")}</h3>
      <p>{t("optionsPageDescription")}</p>
    </div>
    <div id="settings-privacy-entry" class="form-panel privacy-entry-panel">
      <div class="section-heading">
        <h3>{t("privacyLocalDataTitle")}</h3>
        <p>{t("privacyLocalDataEntryDescription")}</p>
      </div>
      <button type="button" class="privacy-entry-action" onclick={onOpenPrivacyPage}>
        <ShieldCheck size={16} />{t("privacyViewAndClear")}
      </button>
    </div>
    <div id="settings-output" class="form-panel">
      <div class="section-heading"><h3>{t("usageModeTitle")}</h3><p>{t("usageModeDescription")}</p></div>
      <div class="form-grid">
        <label class:field-invalid={Boolean(fieldError("hotkey") || hotkeyValidationMessage)}>
          <span>{t("hotkey")}</span>
          <button
            type="button"
            class:recording={hotkeyCaptureState === "recording"}
            class="hotkey-recorder"
            onkeydown={onHotkeyKeydown}
            onclick={onBeginHotkeyCapture}
          >
            <Keyboard size={16} />
            <strong>{hotkeyCaptureState === "recording" ? t("hotkeyRecording") : formatHotkey(config.hotkey) || t("hotkeyUnset")}</strong>
          </button>
          <small class="field-hint">{hotkeyValidationMessage || fieldError("hotkey") || t("hotkeyRecordHint")}</small>
        </label>
      </div>
      <div class="subsection-heading">
        <strong>{t("backupTriggers")}</strong>
        <span>{t("backupTriggersDescription")}</span>
      </div>
      <div class="toggle-grid">
        <label class="check"><input type="checkbox" bind:checked={config.triggers.middle_mouse_enabled} onchange={(event) => onOptionEnabledNotice("middle_mouse_enabled", event.currentTarget.checked)} /><span class="check-copy"><span>{t("middleMouse")}</span><small>{t("tagConflictRisk")}</small></span></label>
        <label class="check"><input type="checkbox" bind:checked={config.triggers.right_alt_enabled} onchange={(event) => onOptionEnabledNotice("right_alt_enabled", event.currentTarget.checked)} /><span class="check-copy"><span>{t("rightAlt")}</span><small>{t("tagConflictRisk")}</small></span></label>
      </div>
      <p class="field-hint">{t("triggerConflictHint")}</p>
    </div>
    <div id="settings-window" class="form-panel">
      <div class="section-heading"><h3>{t("windowBehaviorTitle")}</h3><p>{t("windowBehaviorDescription")}</p></div>
      <div class="form-grid">
        <label class:field-invalid={Boolean(fieldError("tray.close_behavior"))}>
          <span>{t("closeBehavior")}</span>
          <select bind:value={config.tray.close_behavior}>
            <option value="close_to_tray">{t("closeBehaviorCloseToTray")}</option>
            <option value="direct_exit">{t("closeBehaviorDirectExit")}</option>
            <option value="ask_every_time">{t("closeBehaviorAskEveryTime")}</option>
          </select>
          {#if fieldError("tray.close_behavior")}<small class="field-error">{fieldError("tray.close_behavior")}</small>{/if}
        </label>
      </div>
      <div class="toggle-grid">
        <label class="check"><input type="checkbox" bind:checked={config.startup.launch_on_startup} />{t("launchOnStartup")}</label>
      </div>
      <p class="field-hint">{t("closeBehaviorHint")}</p>
    </div>
    <div id="settings-basic-output" class="form-panel">
      <div class="section-heading"><h3>{t("inputResultTitle")}</h3><p>{t("inputResultDescription")}</p></div>
      <div class="form-grid">
        <label class:field-invalid={Boolean(fieldError("typing.paste_method"))}>
          <span>{t("pasteMethod")}</span>
          <select bind:value={config.typing.paste_method}>
            <option value="ctrl_v">Ctrl + V</option>
            <option value="shift_insert">Shift + Insert</option>
            <option value="clipboard_only">{t("clipboardOnly")}</option>
          </select>
          {#if fieldError("typing.paste_method")}<small class="field-error">{fieldError("typing.paste_method")}</small>{/if}
        </label>
      </div>
      <div class="toggle-grid">
        <label class="check"><input type="checkbox" bind:checked={config.typing.remove_trailing_period} />{t("removeTrailingPeriod")}</label>
        <label class="check"><input type="checkbox" bind:checked={config.typing.restore_clipboard_after_paste} />{t("restoreClipboardAfterPaste")}</label>
      </div>
      <p class="field-hint">{t("removeTrailingPeriodHint")}</p>
      <p class="field-hint">{t("clipboardTextRestoreHint")}</p>
    </div>
    <div id="settings-screen-context" class="form-panel">
      <div class="section-heading"><h3>{t("screenContextTitle")}</h3><p>{t("screenContextDescription")}</p></div>
      <div class="toggle-grid">
        <label class="check">
          <input type="checkbox" bind:checked={config.screen_context.enabled} />
          <span class="check-copy">
            <span>{t("enableScreenContext")}</span>
            <small>{t("screenContextSentToService")}</small>
          </span>
        </label>
      </div>
      <p class="field-hint">{t("screenContextPrivacyHint")}</p>
      <div class="update-card">
        <div>
          <strong>{t("screenContextTestTitle")}</strong>
          <p>{t("screenContextTestDescription")}</p>
          {#if screenContextTestResult}
            <small>{t("screenContextTestMeta", {
              chars: String(screenContextTestResult.text_chars),
              ms: String(screenContextTestResult.elapsed_ms),
              lang: screenContextTestResult.selected_language ?? "-"
            })}</small>
          {/if}
        </div>
        <div class="update-actions">
          <button type="button" onclick={onTestScreenContext} disabled={testingScreenContext}>
            <ScanText size={16} />{testingScreenContext ? t("testingScreenContext") : t("testScreenContext")}
          </button>
        </div>
      </div>
      {#if screenContextTestResult}
        <div class="ocr-preview" class:empty={!screenContextTestResult.text.trim()}>
          <strong>{screenContextTestResult.warning ?? t("screenContextRecognizedText")}</strong>
          <pre>{screenContextTestResult.text || t("screenContextNoText")}</pre>
        </div>
      {/if}
    </div>
    <div id="settings-overlay" class="form-panel">
      <div class="section-heading"><h3>{t("floatingCaptionAppearance")}</h3><p>{t("floatingCaptionAppearanceDescription")}</p></div>
      <div class="caption-theme-panel">
        <div class="caption-theme-head">
          <div>
            <strong>{t("captionColors")}</strong>
            <span>{t("captionColorsDescription")}</span>
          </div>
          <div class="caption-preview" style={`--preview-bg-rgb: ${overlayBackgroundRgb()}; --preview-opacity: ${overlayOpacity()}; --preview-text: ${overlayTextColor()};`}>
            {t("captionPreviewText")}
          </div>
        </div>
        <div class="preset-row">
          {#each overlayColorPresets as preset}
            <button
              type="button"
              class:active={overlayPresetActive(preset.background, preset.text)}
              aria-pressed={overlayPresetActive(preset.background, preset.text)}
              onclick={() => onApplyOverlayPreset(preset.background, preset.text)}
            >
              <span class="preset-swatch" style={`--preset-bg: ${preset.background}; --preset-text: ${preset.text};`}>Aa</span>
              <span>{t(preset.label)}</span>
            </button>
          {/each}
        </div>
        <div class="caption-opacity-row" class:field-invalid={Boolean(fieldError("ui.opacity"))}>
          <div>
            <strong>{t("captionOpacity")}</strong>
            <span>{t("captionOpacityDescription")}</span>
          </div>
          <div class="preset-row opacity-preset-row">
            {#each overlayOpacityPresets as opacity}
              <button
                type="button"
                class:active={overlayOpacityPresetActive(opacity)}
                aria-pressed={overlayOpacityPresetActive(opacity)}
                onclick={() => onApplyOverlayOpacity(opacity)}
              >
                {overlayOpacityLabel(opacity)}
              </button>
            {/each}
          </div>
          {#if fieldError("ui.opacity")}<small class="field-error">{fieldError("ui.opacity")}</small>{/if}
        </div>
      </div>
    </div>
    <div id="settings-audio" class="form-panel">
      <div class="section-heading"><h3>{t("microphoneTitle")}</h3><p>{t("microphoneDescription")}</p></div>
      <div class="form-grid">
        <label>
          <span>{t("inputDevice")}</span>
          <select value={config.audio.input_device ?? ""} onchange={(event) => onSetInputDevice(event.currentTarget.value)}>
            <option value="">{t("defaultInputDevice")}</option>
            {#if audioDevices.length === 0}
              <option value="" disabled>{t("noAudioDevices")}</option>
            {/if}
            {#each audioDevices as device}
              <option value={device.index}>{device.index}: {device.name}</option>
            {/each}
          </select>
        </label>
      </div>
    </div>
    <div id="settings-recording-troubleshooting" class="form-panel">
      <div class="section-heading"><h3>{t("recordingTroubleshooting")}</h3><p>{t("recordingTroubleshootingDescription")}</p></div>
      <div class="form-grid">
        <label class:field-invalid={Boolean(fieldError("audio.silence_auto_stop_seconds"))}>
          <span>{t("silenceAutoStopSeconds")}</span>
          <input type="number" min="0" max="300" step="1" bind:value={config.audio.silence_auto_stop_seconds} />
          {#if fieldError("audio.silence_auto_stop_seconds")}<small class="field-error">{fieldError("audio.silence_auto_stop_seconds")}</small>{/if}
        </label>
      </div>
      <p class="field-hint">{t("silenceAutoStopHint")}</p>
    </div>
    <div id="settings-update" class="form-panel update-panel">
      <div class="section-heading"><h3>{t("updatesAndDiagnostics")}</h3><p>{t("updatesAndDiagnosticsDescription")}</p></div>
      <div class:available={updateStatus?.update_available} class="update-card">
        <div>
          <strong>{updatePanelTitle()}</strong>
          <p>{updatePanelDescription()}</p>
          <small>{updateMetaText()}</small>
        </div>
        <div class="update-actions">
          <button type="button" onclick={() => onCheckUpdate(true)} disabled={checkingUpdate}>
            <ShieldCheck size={16} />{checkingUpdate ? t("checkingUpdates") : t("checkUpdates")}
          </button>
          {#if updateStatus?.update_available && updateStatus.asset_name}
            <button type="button" class="primary" onclick={onDownloadLatestUpdate} disabled={installingUpdate}>
              <Download size={16} />{installingUpdate ? t("downloadingInstall") : t("updateNow")}
            </button>
          {/if}
        </div>
      </div>
      <div class="toggle-grid">
        <label class="check"><input type="checkbox" bind:checked={config.update.auto_check_on_startup} />{t("autoCheckUpdates")}</label>
      </div>
    </div>
    <div id="settings-diagnostics" class="form-panel">
      <div class="section-heading"><h3>{t("diagnosticsAndLogs")}</h3><p>{t("diagnosticsDescription")}</p></div>
      <div class="update-card">
        <div>
          <strong>{t("logStatusTitle")}</strong>
          <p>{t("logStatusDescription")}</p>
        </div>
        <div class="update-actions">
          <button type="button" onclick={onOpenLog} disabled={openingLog}>
            <FileText size={16} />{openingLog ? t("openingLog") : t("openLog")}
          </button>
          <button type="button" onclick={onCopyDiagnosticReport} disabled={copyingDiagnosticReport}>
            <ClipboardCopy size={16} />{copyingDiagnosticReport ? t("copyingReport") : t("copyDiagnosticReport")}
          </button>
        </div>
      </div>
    </div>
  </section>
</section>

<style>
  .settings-stack {
    display: grid;
    gap: 18px;
    max-width: 1040px;
  }

  .settings-group {
    display: grid;
    gap: 12px;
  }

  .settings-group-heading {
    display: grid;
    gap: 4px;
    padding: 0 2px;
  }

  .settings-group-heading h3,
  .section-heading h3 {
    margin: 0;
    color: var(--text-main);
    font-weight: 800;
  }

  .settings-group-heading h3 {
    font-size: 20px;
  }

  .settings-group-heading p,
  .section-heading p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .form-panel {
    display: grid;
    gap: 14px;
    min-width: 0;
    padding: 18px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 18px;
  }

  .privacy-entry-panel {
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
  }

  .privacy-entry-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    min-height: 36px;
    min-width: 132px;
    padding: 0 12px;
    color: #ffffff;
    background: var(--primary);
    border: 1px solid var(--primary);
    border-radius: 10px;
    font-weight: 800;
    line-height: 1.2;
    white-space: normal;
    overflow-wrap: anywhere;
  }

  .form-panel[id^="settings-"] {
    scroll-margin-top: 86px;
  }

  .form-panel label {
    display: grid;
    align-content: start;
    gap: 8px;
    color: var(--text-secondary);
    font-size: 14px;
  }

  .section-heading {
    display: grid;
    gap: 4px;
  }

  .subsection-heading {
    display: grid;
    gap: 3px;
    padding-top: 2px;
  }

  .subsection-heading strong {
    color: var(--text-main);
    font-size: 14px;
    font-weight: 800;
  }

  .subsection-heading span {
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.45;
  }

  .section-heading h3 {
    margin-bottom: 6px;
    font-size: 16px;
  }

  .form-grid,
  .toggle-grid {
    display: grid;
    align-items: start;
    gap: 16px 14px;
  }

  .form-grid {
    grid-template-columns: repeat(auto-fit, minmax(min(320px, 100%), 1fr));
  }

  .toggle-grid {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px 18px;
  }

  .check {
    display: inline-flex !important;
    align-items: center;
    flex: 0 1 auto;
    gap: 10px;
    min-height: 38px;
    width: fit-content;
    max-width: 100%;
    min-width: 0;
    color: var(--text-main);
    font-weight: 700;
    line-height: 1.35;
    white-space: normal;
    overflow-wrap: anywhere;
  }

  .check input {
    flex: 0 0 auto;
    width: 18px;
    min-height: 18px;
    accent-color: var(--primary);
  }

  .check-copy {
    display: grid;
    gap: 2px;
    min-width: 0;
  }

  .check-copy span {
    color: var(--text-main);
    font-size: 14px;
    font-weight: 700;
    white-space: normal;
  }

  .check-copy small {
    color: var(--text-muted);
    font-size: 11px;
    font-weight: 700;
    line-height: 1.25;
    white-space: normal;
    overflow-wrap: anywhere;
  }

  input,
  select {
    width: 100%;
    min-height: 38px;
    padding: 0 12px;
    color: var(--text-main);
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 10px;
  }

  select,
  input {
    min-width: 0;
    text-overflow: ellipsis;
  }

  input:focus,
  select:focus,
  button:focus-visible {
    border-color: var(--primary);
    box-shadow: 0 0 0 3px rgba(47, 128, 237, 0.14);
  }

  .field-hint {
    margin: 8px 0 0;
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.45;
  }

  .field-invalid input,
  .field-invalid select {
    border-color: var(--danger);
    background: #fff7f7;
  }

  .field-error {
    color: var(--danger);
    font-size: 12px;
    line-height: 1.35;
  }

  .hotkey-recorder {
    display: inline-flex;
    align-items: center;
    justify-content: flex-start;
    gap: 9px;
    width: 100%;
    min-height: 38px;
    padding: 0 12px;
    color: var(--text-main);
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 10px;
    text-align: left;
  }

  .hotkey-recorder strong {
    min-width: 0;
    overflow: hidden;
    color: inherit;
    font-size: 14px;
    font-weight: 800;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .hotkey-recorder :global(svg) {
    flex: 0 0 auto;
    color: var(--primary);
  }

  .hotkey-recorder.recording {
    border-color: var(--primary);
    background: var(--primary-light);
    box-shadow: 0 0 0 3px rgba(47, 128, 237, 0.14);
  }

  .field-invalid .hotkey-recorder {
    border-color: var(--danger);
    background: #fff7f7;
  }

  .caption-theme-panel {
    display: grid;
    gap: 12px;
    padding: 14px;
    background: #f8fbff;
    border: 1px solid var(--border);
    border-radius: 14px;
  }

  .caption-theme-head {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 12px;
  }

  .caption-theme-head > div:first-child {
    display: grid;
    gap: 4px;
    min-width: 0;
  }

  .caption-theme-head strong,
  .caption-opacity-row strong {
    color: var(--text-main);
    font-size: 14px;
    font-weight: 800;
  }

  .caption-theme-head span,
  .caption-opacity-row span {
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.45;
    overflow-wrap: anywhere;
  }

  .caption-preview {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 168px;
    min-height: 36px;
    padding: 0 14px;
    overflow: hidden;
    color: var(--preview-text);
    background: rgba(var(--preview-bg-rgb), var(--preview-opacity));
    border-radius: 10px;
    font-size: 14px;
    font-weight: 700;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .preset-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(132px, 100%), 1fr));
    gap: 10px;
  }

  .preset-row button {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    min-height: 42px;
    padding: 6px 10px;
    color: var(--text-main);
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 10px;
    font-size: 13px;
    font-weight: 700;
  }

  .preset-row button.active {
    border-color: var(--primary);
    box-shadow: 0 0 0 2px rgba(47, 128, 237, 0.12);
  }

  .preset-row button > span:last-child {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: normal;
  }

  .preset-swatch {
    display: inline-flex;
    flex: 0 0 auto;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 26px;
    color: var(--preset-text);
    background: var(--preset-bg);
    border-radius: 8px;
    font-size: 12px;
    font-weight: 800;
  }

  .caption-opacity-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(220px, 0.8fr);
    align-items: center;
    gap: 12px;
  }

  .caption-opacity-row > div:first-child {
    display: grid;
    gap: 4px;
    min-width: 0;
  }

  .opacity-preset-row {
    grid-template-columns: repeat(auto-fit, minmax(min(112px, 100%), 1fr));
  }

  .opacity-preset-row button {
    justify-content: center;
    padding: 6px 8px;
  }

  .update-actions button:disabled {
    cursor: wait;
    opacity: 0.66;
  }

  .update-card {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    padding: 14px;
    background: #f8fbff;
    border: 1px solid var(--border);
    border-radius: 12px;
  }

  .update-card > div:first-child {
    min-width: 0;
  }

  .update-card.available {
    background: #fff7ed;
    border-color: #fed7aa;
  }

  .update-card strong {
    display: block;
    margin-bottom: 4px;
    color: var(--text-main);
    font-size: 15px;
    font-weight: 800;
  }

  .update-card p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.4;
    overflow-wrap: anywhere;
  }

  .update-card small {
    display: block;
    margin-top: 6px;
    color: var(--text-muted);
    font-size: 12px;
  }

  .update-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    justify-content: flex-end;
    min-width: 0;
  }

  .update-actions button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    min-height: 36px;
    min-width: 118px;
    padding: 0 12px;
    color: var(--text-main);
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 10px;
    font-weight: 700;
    line-height: 1.2;
    white-space: normal;
    overflow-wrap: anywhere;
  }

  .ocr-preview {
    display: grid;
    gap: 8px;
    min-width: 0;
    padding: 12px;
    background: #f8fbff;
    border: 1px solid var(--border);
    border-radius: 12px;
  }

  .ocr-preview.empty {
    background: #fff8ed;
  }

  .ocr-preview strong {
    color: var(--text-main);
    font-size: 13px;
    font-weight: 800;
  }

  .ocr-preview pre {
    max-height: 168px;
    margin: 0;
    overflow: auto;
    color: var(--text-secondary);
    font-family: inherit;
    font-size: 12px;
    line-height: 1.5;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .update-actions .primary {
    color: #ffffff;
    background: var(--primary);
    border-color: var(--primary);
  }

  @media (max-width: 920px) {
    .update-card {
      grid-template-columns: 1fr;
      align-items: stretch;
    }

    .update-actions {
      justify-content: stretch;
    }

    .update-actions button {
      flex: 1 1 150px;
    }

    .privacy-entry-panel {
      grid-template-columns: 1fr;
      align-items: stretch;
    }

    .form-grid {
      grid-template-columns: 1fr;
    }

    .preset-row {
      grid-template-columns: repeat(auto-fit, minmax(min(132px, 100%), 1fr));
    }

    .caption-theme-head {
      grid-template-columns: 1fr;
      align-items: stretch;
    }

    .caption-preview {
      width: 100%;
      min-width: 0;
    }
  }

  @media (max-width: 720px) {
    .form-grid,
    .preset-row {
      grid-template-columns: 1fr;
    }

    .toggle-grid {
      display: grid;
      grid-template-columns: 1fr;
    }

    .check {
      width: 100%;
    }
  }
</style>
