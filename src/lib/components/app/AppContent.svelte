<script lang="ts">
  import ApiConfigSection from "$lib/components/pages/ApiConfigSection.svelte";
  import HotwordsSection from "$lib/components/pages/HotwordsSection.svelte";
  import HomeSection from "$lib/components/pages/HomeSection.svelte";
  import HistorySection, {
    type HistoryDayRow,
    type HistorySummaryCard,
  } from "$lib/components/pages/HistorySection.svelte";
  import OptionsSection from "$lib/components/pages/OptionsSection.svelte";
  import PrivacySection from "$lib/components/pages/PrivacySection.svelte";
  import type { SetupStatusItem, SetupStatusWarning } from "$lib/components/overview/SetupStatusCard.svelte";
  import type { CopyKey, UserErrorDetail } from "$lib/i18n";
  import type {
    AppConfig,
    AudioDeviceInfo,
    LastSessionOutcome,
    LocalDataStatus,
    ScreenContextTestResult,
    Section,
    SelectableHotwordCandidate,
    SoftConfigNoticeKey,
    StatsSnapshot,
    UpdateStatus,
    UserErrorAction,
  } from "$lib/types/app";

  type Translate = (key: CopyKey, values?: Record<string, string>) => string;

  type Props = {
    selectedSection: Section;
    config: AppConfig;
    stats: StatsSnapshot;
    autoHotwordCandidates: SelectableHotwordCandidate[];
    t: Translate;
    uiCompact: boolean;
    recording: boolean;
    saving: boolean;
    inputStatus: "idle" | "listening" | "error";
    inputStatusLabel: string;
    inputStatusDesc: string;
    requiresAsrAuth: boolean;
    setupRequiredMessage: () => string;
    activeErrorDetail: UserErrorDetail | null;
    activeErrorActions: UserErrorAction[];
    lastSessionOutcome: LastSessionOutcome;
    sessionBusy: boolean;
    snapshotHotkey: string;
    chineseTypingCharsPerMinute: number;
    configExists: boolean;
    setupChecking: boolean;
    setupStatusReady: boolean;
    setupStatusItems: SetupStatusItem[];
    setupWarnings: SetupStatusWarning[];
    setupWarningCount: number;
    testingAsr: boolean;
    testingLlm: boolean;
    testingScreenContext: boolean;
    screenContextTestResult: ScreenContextTestResult | null;
    hotkeyCaptureState: "idle" | "recording";
    hotkeyValidationMessage: string;
    overlayColorPresets: Array<{ label: CopyKey; background: string; text: string }>;
    overlayOpacityPresets: readonly number[];
    audioDevices: AudioDeviceInfo[];
    updateStatus: UpdateStatus | null;
    checkingUpdate: boolean;
    installingUpdate: boolean;
    openingLog: boolean;
    copyingDiagnosticReport: boolean;
    generatingAutoHotwords: boolean;
    clearingAutoHotwordHistory: boolean;
    autoHotwordError: string;
    showAutoHotwordDetails: boolean;
    hasLlmApiConfig: boolean;
    hotwordCount: number;
    acceptedAutoHotwordCount: number;
    selectedAutoHotwordCount: number;
    autoHotwordStatusText: string;
    llmApiStatusText: string;
    fieldError: (field: string) => string;
    candidateConfidenceLabel: (confidence: number) => string;
    formatHotkey: (value: string) => string;
    formatNumber: (value: number) => string;
    formatHours: (seconds: number) => string;
    formatSavedHours: (hours: number) => string;
    weeklySavedHours: () => number;
    usageTipText: () => string;
    triggerLabel: (enabled: boolean) => string;
    setupActionText: (action: string) => string;
    overlayBackgroundRgb: () => string;
    overlayOpacity: () => number;
    overlayTextColor: () => string;
    overlayPresetActive: (background: string, text: string) => boolean;
    overlayOpacityPresetActive: (value: number) => boolean;
    overlayOpacityLabel: (value: number) => string;
    updatePanelTitle: () => string;
    updatePanelDescription: () => string;
    updateMetaText: () => string;
    historySummaryCards: () => HistorySummaryCard[];
    recentSevenDayDisplayRows: () => HistoryDayRow[];
    privacyStatus: LocalDataStatus | null;
    privacyClearingRecentContext: boolean;
    privacyClearingAutoHotwordHistory: boolean;
    privacyClearingUsageStats: boolean;
    onOpenSettings: () => void;
    onOpenSetupGuide: () => void;
    onUserErrorAction: (action: UserErrorAction) => void;
    onCopyLastOutcomeText: (text: string) => Promise<boolean>;
    onToggleRecording: () => void;
    onSelectSection: (section: Section) => void;
    onUpdateHotwords: (value: string) => void;
    onTidyHotwords: () => void;
    onClearHotwords: () => void;
    onUpdatePromptContext: (value: string) => void;
    onOptionEnabledNotice: (key: SoftConfigNoticeKey, enabled: boolean) => void;
    onRestoreDefaultPrompt: () => void;
    onPreviewFinalPrompt: () => void;
    onOpenLlmApiSettings: () => void;
    onGenerateAutoHotwords: () => void;
    onClearAutoHotwordHistory: () => void;
    onRefreshAutoHotwordStatus: () => void;
    onUpdateAcceptedAutoHotwords: (value: string) => void;
    onTidyAcceptedAutoHotwords: () => void;
    onClearAcceptedAutoHotwords: () => void;
    onApplySelectedAutoHotwords: () => void;
    onScrollToSettingsPanel: (id: string) => void;
    onRefreshSetupStatus: () => void;
    onSetupAction: (action: string) => void;
    onOpenDoubaoAsrDocs: () => void;
    onTestAsrConfig: () => void;
    onTestLlmConfig: () => void;
    onTestScreenContext: () => void;
    onHotkeyKeydown: (event: KeyboardEvent) => void;
    onBeginHotkeyCapture: () => void;
    onApplyOverlayPreset: (background: string, text: string) => void;
    onApplyOverlayOpacity: (value: number) => void;
    onSetInputDevice: (value: string | number | null) => void;
    onCheckUpdate: (manual?: boolean) => void;
    onDownloadLatestUpdate: () => void;
    onOpenLog: () => void;
    onCopyDiagnosticReport: () => void;
    onRefreshPrivacyStatus: () => void;
    onClearPrivacyRecentContext: () => void;
    onClearPrivacyAutoHotwordHistory: () => void;
    onClearPrivacyUsageStats: () => void;
  };

  let {
    selectedSection,
    config = $bindable<AppConfig>(),
    stats,
    autoHotwordCandidates = $bindable<SelectableHotwordCandidate[]>(),
    t,
    uiCompact,
    recording,
    saving,
    inputStatus,
    inputStatusLabel,
    inputStatusDesc,
    requiresAsrAuth,
    setupRequiredMessage,
    activeErrorDetail,
    activeErrorActions,
    lastSessionOutcome,
    sessionBusy,
    snapshotHotkey,
    chineseTypingCharsPerMinute,
    configExists,
    setupChecking,
    setupStatusReady,
    setupStatusItems,
    setupWarnings,
    setupWarningCount,
    testingAsr,
    testingLlm,
    testingScreenContext,
    screenContextTestResult,
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
    generatingAutoHotwords,
    clearingAutoHotwordHistory,
    autoHotwordError,
    showAutoHotwordDetails,
    hasLlmApiConfig,
    hotwordCount,
    acceptedAutoHotwordCount,
    selectedAutoHotwordCount,
    autoHotwordStatusText,
    llmApiStatusText,
    fieldError,
    candidateConfidenceLabel,
    formatHotkey,
    formatNumber,
    formatHours,
    formatSavedHours,
    weeklySavedHours,
    usageTipText,
    triggerLabel,
    setupActionText,
    overlayBackgroundRgb,
    overlayOpacity,
    overlayTextColor,
    overlayPresetActive,
    overlayOpacityPresetActive,
    overlayOpacityLabel,
    updatePanelTitle,
    updatePanelDescription,
    updateMetaText,
    historySummaryCards,
    recentSevenDayDisplayRows,
    privacyStatus,
    privacyClearingRecentContext,
    privacyClearingAutoHotwordHistory,
    privacyClearingUsageStats,
    onOpenSettings,
    onOpenSetupGuide,
    onUserErrorAction,
    onCopyLastOutcomeText,
    onToggleRecording,
    onSelectSection,
    onUpdateHotwords,
    onTidyHotwords,
    onClearHotwords,
    onUpdatePromptContext,
    onOptionEnabledNotice,
    onRestoreDefaultPrompt,
    onPreviewFinalPrompt,
    onOpenLlmApiSettings,
    onGenerateAutoHotwords,
    onClearAutoHotwordHistory,
    onRefreshAutoHotwordStatus,
    onUpdateAcceptedAutoHotwords,
    onTidyAcceptedAutoHotwords,
    onClearAcceptedAutoHotwords,
    onApplySelectedAutoHotwords,
    onScrollToSettingsPanel,
    onRefreshSetupStatus,
    onSetupAction,
    onOpenDoubaoAsrDocs,
    onTestAsrConfig,
    onTestLlmConfig,
    onTestScreenContext,
    onHotkeyKeydown,
    onBeginHotkeyCapture,
    onApplyOverlayPreset,
    onApplyOverlayOpacity,
    onSetInputDevice,
    onCheckUpdate,
    onDownloadLatestUpdate,
    onOpenLog,
    onCopyDiagnosticReport,
    onRefreshPrivacyStatus,
    onClearPrivacyRecentContext,
    onClearPrivacyAutoHotwordHistory,
    onClearPrivacyUsageStats,
  }: Props = $props();
</script>

{#if selectedSection === "Home"}
  <HomeSection
    {config}
    {stats}
    {t}
    {uiCompact}
    {recording}
    {saving}
    {inputStatus}
    {inputStatusLabel}
    {inputStatusDesc}
    {requiresAsrAuth}
    setupRequiredMessage={setupRequiredMessage()}
    {activeErrorDetail}
    {activeErrorActions}
    {lastSessionOutcome}
    {onCopyLastOutcomeText}
    {sessionBusy}
    {snapshotHotkey}
    {chineseTypingCharsPerMinute}
    {formatHotkey}
    {formatNumber}
    {formatHours}
    {formatSavedHours}
    {weeklySavedHours}
    {usageTipText}
    {triggerLabel}
    onOpenSettings={onOpenSettings}
    onOpenSetupGuide={onOpenSetupGuide}
    onUserErrorAction={onUserErrorAction}
    onToggleRecording={onToggleRecording}
    onSelectSection={onSelectSection}
  />
{:else if selectedSection === "Hotwords"}
  <HotwordsSection
    bind:config
    bind:autoHotwordCandidates
    {t}
    {generatingAutoHotwords}
    {clearingAutoHotwordHistory}
    {autoHotwordError}
    {showAutoHotwordDetails}
    {hasLlmApiConfig}
    {hotwordCount}
    {acceptedAutoHotwordCount}
    {selectedAutoHotwordCount}
    {autoHotwordStatusText}
    {fieldError}
    {candidateConfidenceLabel}
    onUpdateHotwords={onUpdateHotwords}
    onTidyHotwords={onTidyHotwords}
    onClearHotwords={onClearHotwords}
    onUpdatePromptContext={onUpdatePromptContext}
    onOptionEnabledNotice={onOptionEnabledNotice}
    onRestoreDefaultPrompt={onRestoreDefaultPrompt}
    onPreviewFinalPrompt={onPreviewFinalPrompt}
    onOpenLlmApiSettings={onOpenLlmApiSettings}
    onGenerateAutoHotwords={onGenerateAutoHotwords}
    onClearAutoHotwordHistory={onClearAutoHotwordHistory}
    onRefreshAutoHotwordStatus={onRefreshAutoHotwordStatus}
    onUpdateAcceptedAutoHotwords={onUpdateAcceptedAutoHotwords}
    onTidyAcceptedAutoHotwords={onTidyAcceptedAutoHotwords}
    onClearAcceptedAutoHotwords={onClearAcceptedAutoHotwords}
    onApplySelectedAutoHotwords={onApplySelectedAutoHotwords}
  />
{:else if selectedSection === "ApiConfig"}
  <ApiConfigSection
    bind:config
    {t}
    {configExists}
    {setupChecking}
    {setupStatusReady}
    {setupStatusItems}
    {setupWarnings}
    {setupWarningCount}
    {snapshotHotkey}
    {requiresAsrAuth}
    {testingAsr}
    {testingLlm}
    {hasLlmApiConfig}
    {llmApiStatusText}
    {fieldError}
    {setupRequiredMessage}
    {setupActionText}
    {formatHotkey}
    onScrollToSettingsPanel={onScrollToSettingsPanel}
    onOpenSetupGuide={onOpenSetupGuide}
    onOpenDoubaoAsrDocs={onOpenDoubaoAsrDocs}
    onRefreshSetupStatus={onRefreshSetupStatus}
    onSetupAction={onSetupAction}
    onTestAsrConfig={onTestAsrConfig}
    onTestLlmConfig={onTestLlmConfig}
  />
{:else if selectedSection === "Options"}
  <OptionsSection
    bind:config
    {t}
    {hotkeyCaptureState}
    {hotkeyValidationMessage}
    {overlayColorPresets}
    {overlayOpacityPresets}
    {audioDevices}
    {updateStatus}
    {checkingUpdate}
    {installingUpdate}
    {openingLog}
    {copyingDiagnosticReport}
    {testingScreenContext}
    {screenContextTestResult}
    {fieldError}
    {formatHotkey}
    {overlayBackgroundRgb}
    {overlayOpacity}
    {overlayTextColor}
    {overlayPresetActive}
    {overlayOpacityPresetActive}
    {overlayOpacityLabel}
    {updatePanelTitle}
    {updatePanelDescription}
    {updateMetaText}
    onHotkeyKeydown={onHotkeyKeydown}
    onBeginHotkeyCapture={onBeginHotkeyCapture}
    onOptionEnabledNotice={onOptionEnabledNotice}
    onApplyOverlayPreset={onApplyOverlayPreset}
    onApplyOverlayOpacity={onApplyOverlayOpacity}
    onSetInputDevice={onSetInputDevice}
    onCheckUpdate={onCheckUpdate}
    onDownloadLatestUpdate={onDownloadLatestUpdate}
    onOpenLog={onOpenLog}
    onCopyDiagnosticReport={onCopyDiagnosticReport}
    onTestScreenContext={onTestScreenContext}
  />
{:else if selectedSection === "Privacy"}
  <PrivacySection
    bind:config
    {t}
    status={privacyStatus}
    clearingRecentContext={privacyClearingRecentContext}
    clearingAutoHotwordHistory={privacyClearingAutoHotwordHistory}
    clearingUsageStats={privacyClearingUsageStats}
    {hasLlmApiConfig}
    onRefreshStatus={onRefreshPrivacyStatus}
    onOpenLlmApiSettings={onOpenLlmApiSettings}
    onOptionEnabledNotice={onOptionEnabledNotice}
    onClearRecentContext={onClearPrivacyRecentContext}
    onClearAutoHotwordHistory={onClearPrivacyAutoHotwordHistory}
    onClearUsageStats={onClearPrivacyUsageStats}
  />
{:else if selectedSection === "History"}
  <HistorySection
    summaryCards={historySummaryCards()}
    dayRows={recentSevenDayDisplayRows()}
    byDayTitle={t("byDay")}
    byDayDescription={t("lastSevenDays")}
    dateColumnLabel={t("dateColumn")}
    inputCharsLabel={t("dailyInputChars")}
    voiceDurationLabel={t("voiceDuration")}
    averageSpeedLabel={t("averageInputSpeed")}
    savedTimeLabel={t("dailySavedTime")}
  />
{/if}
