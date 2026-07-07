<script lang="ts">
  import type { Snippet } from "svelte";
  import { ChevronDown } from "lucide-svelte";

  type Props = {
    title: string;
    description: string;
    expanded: boolean;
    // rootId is for navigation targets; panelId is for aria-controls.
    rootId?: string;
    panelId?: string;
    onToggle: () => void;
    children?: Snippet;
  };

  let { title, description, expanded, rootId, panelId, onToggle, children }: Props = $props();
</script>

<div id={rootId} class="advanced-settings">
  <button
    type="button"
    class="advanced-toggle"
    aria-expanded={expanded}
    aria-controls={panelId}
    onclick={onToggle}
  >
    <span>
      <strong>{title}</strong>
      <small>{description}</small>
    </span>
    <ChevronDown size={16} class={expanded ? "expanded" : ""} />
  </button>
  {#if expanded}
    <div id={panelId} class="advanced-panel">
      {@render children?.()}
    </div>
  {/if}
</div>

<style>
  .advanced-settings {
    display: grid;
    gap: 10px;
    min-width: 0;
  }

  .advanced-toggle {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 10px;
    width: 100%;
    min-height: 44px;
    padding: 10px 12px;
    color: var(--text-main);
    background: #f8fbff;
    border: 1px solid var(--border);
    border-radius: 10px;
    text-align: left;
  }

  .advanced-toggle span {
    display: grid;
    gap: 3px;
    min-width: 0;
  }

  .advanced-toggle strong {
    font-size: 13px;
    font-weight: 800;
  }

  .advanced-toggle small {
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.35;
    overflow-wrap: anywhere;
  }

  .advanced-toggle :global(svg) {
    color: var(--text-secondary);
    transition: transform 0.16s ease;
  }

  .advanced-toggle :global(svg.expanded) {
    transform: rotate(180deg);
  }

  .advanced-panel {
    display: grid;
    gap: 12px;
    min-width: 0;
    padding: 12px;
    background: #fbfdff;
    border: 1px solid var(--border);
    border-radius: 10px;
  }
</style>
