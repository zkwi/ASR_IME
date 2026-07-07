<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    title: string;
    description: string;
    meta?: string;
    tone?: "default" | "info" | "warning" | "available";
    available?: boolean;
    actionVisible?: boolean;
    ariaLive?: "off" | "polite" | "assertive";
    actions?: Snippet;
  };

  let {
    title,
    description,
    meta = "",
    tone = "default",
    available = false,
    actionVisible = true,
    ariaLive = "off",
    actions,
  }: Props = $props();
</script>

<section
  class:available={available || tone === "available"}
  class:info={tone === "info"}
  class:warning={tone === "warning"}
  class="action-panel"
  aria-live={ariaLive}
>
  <div class="action-copy">
    <strong>{title}</strong>
    <p>{description}</p>
    {#if meta}
      <small>{meta}</small>
    {/if}
  </div>
  {#if actions && actionVisible}
    <div class="action-panel-actions">
      {@render actions()}
    </div>
  {/if}
</section>

<style>
  .action-panel {
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

  .action-panel.available {
    background: #fff7ed;
    border-color: #fed7aa;
  }

  .action-panel.info {
    background: #f7fbff;
    border-color: rgba(47, 128, 237, 0.16);
  }

  .action-panel.warning {
    background: #fffaf3;
    border-color: rgba(217, 119, 6, 0.26);
  }

  .action-copy {
    min-width: 0;
  }

  .action-copy strong {
    display: block;
    margin-bottom: 4px;
    color: var(--text-main);
    font-size: 15px;
    font-weight: 800;
  }

  .action-panel.warning .action-copy strong {
    color: #8a4b00;
  }

  .action-copy p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.4;
    overflow-wrap: anywhere;
  }

  .action-copy small {
    display: block;
    margin-top: 6px;
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.35;
    overflow-wrap: anywhere;
  }

  .action-panel-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    justify-content: flex-end;
    min-width: 0;
  }

  .action-panel-actions :global(button) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    min-width: 118px;
    min-height: 36px;
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

  .action-panel-actions :global(button:disabled) {
    cursor: wait;
    opacity: 0.66;
  }

  .action-panel-actions :global(.primary) {
    color: #ffffff;
    background: var(--primary);
    border-color: var(--primary);
  }

  @media (max-width: 920px) {
    .action-panel {
      grid-template-columns: 1fr;
      align-items: stretch;
    }

    .action-panel-actions {
      justify-content: stretch;
    }

    .action-panel-actions :global(button) {
      flex: 1 1 150px;
    }
  }
</style>
