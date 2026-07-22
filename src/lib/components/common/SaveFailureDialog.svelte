<script lang="ts">
  import { focusTrap } from "$lib/utils/focusTrap";
  type Props = {
    visible: boolean;
    title: string;
    body: string;
    error: string;
    retryLabel: string;
    discardLabel: string;
    cancelLabel: string;
    saving: boolean;
    onRetry: () => void;
    onDiscard: () => void;
    onCancel: () => void;
  };

  let {
    visible,
    title,
    body,
    error,
    retryLabel,
    discardLabel,
    cancelLabel,
    saving,
    onRetry,
    onDiscard,
    onCancel,
  }: Props = $props();
</script>

{#if visible}
  <div class="modal-backdrop" role="presentation">
    <div class="save-failure-dialog" role="dialog" aria-modal="true" aria-labelledby="save-failure-title" aria-describedby="save-failure-body" tabindex="-1" use:focusTrap={{ initialFocus: ".primary", onEscape: onCancel }}>
      <div>
        <h3 id="save-failure-title">{title}</h3>
        <p id="save-failure-body">{body}</p>
        {#if error}<p class="error-detail" role="alert">{error}</p>{/if}
      </div>
      <div class="dialog-actions">
        <button type="button" class="primary" disabled={saving} onclick={onRetry}>{retryLabel}</button>
        <button type="button" class="danger" disabled={saving} onclick={onDiscard}>{discardLabel}</button>
        <button type="button" disabled={saving} onclick={onCancel}>{cancelLabel}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 40;
    display: grid;
    place-items: center;
    padding: 24px;
    background: rgba(15, 23, 42, 0.3);
  }

  .save-failure-dialog {
    display: grid;
    gap: 18px;
    width: min(460px, 100%);
    padding: 20px;
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 16px;
    box-shadow: 0 20px 60px rgba(15, 23, 42, 0.2);
  }

  h3 {
    margin: 0 0 8px;
    color: var(--text-main);
    font-size: 18px;
    font-weight: 800;
  }

  p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 14px;
    line-height: 1.55;
  }

  .error-detail {
    margin-top: 10px;
    padding: 9px 11px;
    color: #991b1b;
    background: #fff5f5;
    border: 1px solid rgba(239, 68, 68, 0.24);
    border-radius: 10px;
    overflow-wrap: anywhere;
  }

  .dialog-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 10px;
  }

  button {
    min-height: 36px;
    padding: 0 13px;
    color: var(--text-main);
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 10px;
    font-weight: 700;
  }

  button.primary {
    color: #ffffff;
    background: var(--primary);
    border-color: var(--primary);
  }

  button.danger {
    color: #b91c1c;
    background: #fff5f5;
    border-color: rgba(239, 68, 68, 0.26);
  }
</style>
