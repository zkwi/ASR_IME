<script lang="ts">
  import { focusTrap } from "$lib/utils/focusTrap";
  type Props = {
    visible: boolean;
    title: string;
    body: string;
    gotItLabel: string;
    dontShowAgainLabel: string;
    exitLabel: string;
    onConfirm: () => void;
    onDontShowAgain: () => void;
    onExit: () => void;
  };

  let {
    visible,
    title,
    body,
    gotItLabel,
    dontShowAgainLabel,
    exitLabel,
    onConfirm,
    onDontShowAgain,
    onExit,
  }: Props = $props();
</script>

{#if visible}
  <div class="modal-backdrop" role="presentation">
    <div class="close-prompt" role="dialog" aria-modal="true" aria-labelledby="close-prompt-title" tabindex="-1" use:focusTrap={{ initialFocus: ".primary", onEscape: onConfirm }}>
      <div>
        <h3 id="close-prompt-title">{title}</h3>
        <p>{body}</p>
      </div>
      <div class="close-prompt-actions">
        <button class="primary" onclick={onConfirm}>{gotItLabel}</button>
        <button onclick={onDontShowAgain}>{dontShowAgainLabel}</button>
        <button class="danger" onclick={onExit}>{exitLabel}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 30;
    display: grid;
    place-items: center;
    padding: 24px;
    background: rgba(15, 23, 42, 0.26);
  }

  .close-prompt {
    display: grid;
    gap: 18px;
    width: min(420px, 100%);
    padding: 20px;
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 16px;
    box-shadow: 0 20px 60px rgba(15, 23, 42, 0.2);
  }

  .close-prompt h3 {
    margin: 0 0 8px;
    color: var(--text-main);
    font-size: 18px;
    font-weight: 800;
  }

  .close-prompt p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 14px;
    line-height: 1.55;
  }

  .close-prompt-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 10px;
  }

  .close-prompt-actions button {
    min-height: 36px;
    padding: 0 13px;
    color: var(--text-main);
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 10px;
    font-weight: 700;
  }

  .close-prompt-actions .primary {
    color: #ffffff;
    background: var(--primary);
    border-color: var(--primary);
  }

  .close-prompt-actions .danger {
    color: #b91c1c;
    background: #fff5f5;
    border-color: rgba(239, 68, 68, 0.26);
  }
</style>
