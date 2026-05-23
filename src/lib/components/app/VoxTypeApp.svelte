<script lang="ts">
  import { overlayMeterBars } from "$lib/app/defaults";
  import { createVoxTypeController } from "$lib/app/VoxTypeController.svelte";
  import AppContent from "$lib/components/app/AppContent.svelte";
  import AppGlobalStyles from "$lib/components/app/AppGlobalStyles.svelte";
  import AppShell from "$lib/components/app/AppShell.svelte";
  import ActionNotice from "$lib/components/common/ActionNotice.svelte";
  import CloseToTrayDialog from "$lib/components/common/CloseToTrayDialog.svelte";
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
    <AppContent
      bind:config={app.config}
      bind:autoHotwordCandidates={app.autoHotwordCandidates}
      {...app.appContentProps()}
    />
  </AppShell>

  <ActionNotice
    message={app.actionNotice}
    kind={app.actionNoticeKind}
    actionLabel={app.actionNoticeActionLabel}
    actionBusyLabel={app.actionNoticeActionBusyLabel}
    actionBusy={app.actionNoticeActionBusy}
    onAction={app.runActionNoticeAction}
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
{/if}
