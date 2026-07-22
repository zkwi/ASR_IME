<script lang="ts">
  import { AlertCircle, Check, Download, Info, X } from "lucide-svelte";
  import { noticeRole } from "$lib/utils/notificationPolicy";

  export type ActionNoticeKind = "success" | "info" | "warning" | "error";

  type Props = {
    message?: string | null;
    kind?: ActionNoticeKind;
    actionLabel?: string;
    actionBusyLabel?: string;
    actionBusy?: boolean;
    onAction?: () => void | Promise<void>;
    closeLabel: string;
    onClose: () => void;
  };

  let {
    message = null,
    kind = "info",
    actionLabel = "",
    actionBusyLabel = "",
    actionBusy = false,
    onAction,
    closeLabel,
    onClose,
  }: Props = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (message && event.key === "Escape" && !event.defaultPrevented) onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if message}
  <div
    class:error={kind === "error"}
    class:info={kind === "info"}
    class:warning={kind === "warning"}
    class="action-notice"
    role={noticeRole(kind)}
    aria-live={kind === "error" ? "assertive" : "polite"}
  >
    <div class="notice-main">
      {#if kind === "success"}
        <Check size={16} />
      {:else if kind === "info"}
        <Info size={16} />
      {:else}
        <AlertCircle size={16} />
      {/if}
      <span>{message}</span>
    </div>
    {#if actionLabel && onAction}
      <button type="button" class="notice-action" disabled={actionBusy} onclick={onAction}>
        <Download size={14} />
        <span>{actionBusy ? actionBusyLabel || actionLabel : actionLabel}</span>
      </button>
    {/if}
    <button type="button" class="notice-close" aria-label={closeLabel} title={closeLabel} onclick={onClose}>
      <X size={14} />
    </button>
  </div>
{/if}

<style>
  .action-notice {
    position: fixed;
    right: 22px;
    bottom: 20px;
    z-index: 20;
    display: inline-flex;
    align-items: center;
    flex-wrap: wrap;
    max-width: min(620px, calc(100vw - 44px));
    min-height: 40px;
    gap: 10px;
    padding: 10px 14px;
    color: #0f5132;
    background: rgba(240, 253, 244, 0.98);
    border: 1px solid rgba(34, 197, 94, 0.26);
    border-radius: 12px;
    box-shadow: 0 14px 34px rgba(15, 23, 42, 0.12);
    font-size: 14px;
    font-weight: 700;
  }

  .notice-main {
    display: inline-flex;
    align-items: flex-start;
    flex: 1 1 280px;
    gap: 8px;
    min-width: 0;
  }

  .notice-main :global(svg) {
    flex: 0 0 auto;
    margin-top: 2px;
  }

  .notice-main span {
    min-width: 0;
    line-height: 1.4;
    overflow-wrap: anywhere;
    white-space: normal;
  }

  .action-notice .notice-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
    min-height: 32px;
    gap: 6px;
    padding: 0 12px;
    color: #ffffff;
    background: #d97706;
    border: 1px solid rgba(180, 83, 9, 0.22);
    border-radius: 999px;
    box-shadow: 0 8px 20px rgba(217, 119, 6, 0.18);
    font-size: 13px;
    font-weight: 800;
    transition: transform 140ms ease, box-shadow 140ms ease, opacity 140ms ease;
  }

  .action-notice .notice-action:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 10px 24px rgba(217, 119, 6, 0.24);
  }

  .action-notice .notice-action:disabled {
    cursor: wait;
    opacity: 0.7;
  }

  .notice-close {
    display: grid;
    width: 30px;
    height: 30px;
    flex: 0 0 30px;
    place-items: center;
    padding: 0;
    color: currentColor;
    background: transparent;
    border: 1px solid color-mix(in srgb, currentColor 24%, transparent);
    border-radius: 999px;
  }

  .notice-close:hover {
    background: color-mix(in srgb, currentColor 9%, transparent);
  }

  .action-notice.info {
    color: #245b93;
    background: rgba(240, 247, 255, 0.98);
    border-color: rgba(47, 128, 237, 0.24);
  }

  .action-notice.warning {
    color: #854d0e;
    background: rgba(255, 251, 235, 0.98);
    border-color: rgba(245, 158, 11, 0.32);
  }

  .action-notice.error {
    color: #991b1b;
    background: rgba(254, 242, 242, 0.98);
    border-color: rgba(239, 68, 68, 0.3);
  }

  @media (prefers-reduced-motion: no-preference) {
    .action-notice {
      animation: action-notice-enter 180ms ease-out;
    }

    @keyframes action-notice-enter {
      from {
        opacity: 0;
        transform: translateY(8px);
      }

      to {
        opacity: 1;
        transform: translateY(0);
      }
    }
  }
</style>
