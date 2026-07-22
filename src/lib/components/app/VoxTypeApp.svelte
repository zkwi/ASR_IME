<script lang="ts">
  import { overlayMeterBars } from "$lib/app/defaults";
  import { createVoxTypeController } from "$lib/app/VoxTypeController.svelte";
  import AppContent from "$lib/components/app/AppContent.svelte";
  import AppGlobalStyles from "$lib/components/app/AppGlobalStyles.svelte";
  import AppShell from "$lib/components/app/AppShell.svelte";
  import ActionNotice from "$lib/components/common/ActionNotice.svelte";
  import CloseToTrayDialog from "$lib/components/common/CloseToTrayDialog.svelte";
  import PromptPreviewDialog from "$lib/components/common/PromptPreviewDialog.svelte";
  import SaveFailureDialog from "$lib/components/common/SaveFailureDialog.svelte";
  import OverlayWindow from "$lib/components/overlay/OverlayWindow.svelte";
  import StartupToast from "$lib/components/overlay/StartupToast.svelte";

  const app = createVoxTypeController();

  function suppressAppContextMenu(event: MouseEvent) {
    const target = event.target;
    if (!(target instanceof Element)) {
      event.preventDefault();
      return;
    }
    if (target.closest("input, textarea, select, [contenteditable]")) return;
    event.preventDefault();
  }
</script>

<svelte:head>
  <title>VoxType</title>
</svelte:head>

<svelte:window oncontextmenu={suppressAppContextMenu} />

<AppGlobalStyles />

{#if app.isOverlay}
  <OverlayWindow
    meterBars={overlayMeterBars}
    displayLines={app.overlayDisplayLines}
    recording={app.recording}
    mode={app.overlayMode}
    fontSize={app.overlayFontSize}
    rootStyle={app.overlayRootStyle}
    meterBarHeight={app.overlayMeterBarHeight}
    meterBarOpacity={app.overlayMeterBarOpacity}
    bind:textElement={app.overlayTextElement}
  />
{:else if app.isToast}
  <StartupToast title={app.toastTitle} hint={app.toastHint} />
{:else}
  <AppShell {...app.appShellProps()}>
    <div class="config-editable-region" inert={!app.configEditable} aria-disabled={!app.configEditable}>
      <AppContent
        bind:config={app.config}
        bind:autoHotwordCandidates={app.autoHotwordCandidates}
        {...app.appContentProps()}
      />
    </div>
  </AppShell>

  <ActionNotice
    message={app.actionNotice}
    kind={app.actionNoticeKind}
    actionLabel={app.actionNoticeActionLabel}
    actionBusyLabel={app.actionNoticeActionBusyLabel}
    actionBusy={app.actionNoticeActionBusy}
    onAction={app.runActionNoticeAction}
    closeLabel={app.actionNoticeCloseLabel}
    onClose={app.closeActionNotice}
  />
  <CloseToTrayDialog
    visible={app.closePromptVisible}
    title={app.closePromptTitle}
    body={app.closePromptBody}
    gotItLabel={app.closePromptGotItLabel}
    dontShowAgainLabel={app.closePromptDontShowAgainLabel}
    exitLabel={app.closePromptExitLabel}
    onConfirm={app.confirmClosePrompt}
    onDontShowAgain={app.closeWindowWithoutFuturePrompt}
    onExit={app.exitFromClosePrompt}
  />
  <SaveFailureDialog
    visible={app.saveFailurePromptVisible}
    title={app.saveFailurePromptTitle}
    body={app.saveFailurePromptBody}
    error={app.saveFailurePromptError}
    retryLabel={app.saveFailureRetryLabel}
    discardLabel={app.saveFailureDiscardLabel}
    cancelLabel={app.saveFailureCancelLabel}
    saving={app.savingConfig}
    onRetry={app.retrySaveAndContinue}
    onDiscard={app.discardAndContinue}
    onCancel={app.cancelSaveFailurePrompt}
  />
  <PromptPreviewDialog
    visible={app.promptPreviewVisible}
    title={app.promptPreviewTitle}
    text={app.promptPreviewText}
    copyLabel={app.promptPreviewCopyLabel}
    closeLabel={app.promptPreviewCloseLabel}
    onCopy={app.copyPromptPreview}
    onClose={app.closePromptPreview}
  />
{/if}

<style>
  .config-editable-region {
    display: contents;
  }
</style>
