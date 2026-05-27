<script lang="ts">
  type Props = {
    visible: boolean;
    title: string;
    text: string;
    copyLabel: string;
    closeLabel: string;
    onCopy: () => void;
    onClose: () => void;
  };

  let { visible, title, text, copyLabel, closeLabel, onCopy, onClose }: Props = $props();

  function handleWindowKeydown(event: KeyboardEvent) {
    if (visible && event.key === "Escape") {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

{#if visible}
  <div class="prompt-preview-backdrop" role="presentation" onmousedown={(event) => event.target === event.currentTarget && onClose()}>
    <div class="prompt-preview-dialog" role="dialog" aria-modal="true" aria-labelledby="prompt-preview-title">
      <div class="prompt-preview-head">
        <h3 id="prompt-preview-title">{title}</h3>
        <button type="button" class="ghost-action" onclick={onClose}>{closeLabel}</button>
      </div>
      <textarea readonly value={text} aria-label={title}></textarea>
      <div class="prompt-preview-actions">
        <button type="button" class="secondary-action" onclick={onCopy}>{copyLabel}</button>
        <button type="button" class="primary-action" onclick={onClose}>{closeLabel}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .prompt-preview-backdrop {
    position: fixed;
    inset: 0;
    z-index: 35;
    display: grid;
    place-items: center;
    padding: 24px;
    background: rgba(15, 23, 42, 0.28);
  }

  .prompt-preview-dialog {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr) auto;
    gap: 14px;
    width: min(860px, 100%);
    max-height: min(680px, calc(100vh - 48px));
    padding: 18px;
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: 0 22px 70px rgba(15, 23, 42, 0.22);
  }

  .prompt-preview-head,
  .prompt-preview-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 10px;
    min-width: 0;
  }

  .prompt-preview-head h3 {
    margin: 0;
    color: var(--text-main);
    font-size: 18px;
    font-weight: 800;
    line-height: 1.35;
  }

  textarea {
    width: 100%;
    min-height: 360px;
    min-width: 0;
    padding: 14px;
    color: var(--text-main);
    background: #f8fbff;
    border: 1px solid var(--border);
    border-radius: 10px;
    font-family: ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace;
    font-size: 12px;
    line-height: 1.55;
    resize: none;
    white-space: pre;
  }

  .prompt-preview-actions {
    justify-content: flex-end;
  }

  .primary-action,
  .secondary-action,
  .ghost-action {
    min-height: 34px;
    padding: 0 12px;
    border: 1px solid var(--border);
    border-radius: 10px;
    font-size: 13px;
    font-weight: 800;
    line-height: 1.2;
  }

  .primary-action {
    color: #ffffff;
    background: var(--primary);
    border-color: var(--primary);
  }

  .secondary-action,
  .ghost-action {
    color: var(--text-main);
    background: #ffffff;
  }

  .ghost-action {
    min-height: 30px;
    color: var(--text-secondary);
  }

  @media (max-width: 720px) {
    .prompt-preview-backdrop {
      padding: 12px;
    }

    .prompt-preview-dialog {
      max-height: calc(100vh - 24px);
      padding: 14px;
    }

    textarea {
      min-height: 320px;
      font-size: 11px;
    }

    .prompt-preview-actions button {
      width: 100%;
    }
  }
</style>
