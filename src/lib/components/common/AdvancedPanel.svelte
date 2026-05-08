<script lang="ts">
  type Props = {
    title: string;
    description: string;
    open: boolean;
    showLabel: string;
    hideLabel: string;
    onToggle: () => void;
  };

  let { title, description, open, showLabel, hideLabel, onToggle }: Props = $props();
</script>

<section class="settings-mode-panel">
  <div>
    <h3>{title}</h3>
    <p>{description}</p>
  </div>
  <button type="button" onclick={onToggle}>
    {open ? hideLabel : showLabel}
  </button>
</section>

<style>
  .settings-mode-panel {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 14px;
    padding: 14px 16px;
    background: #f8fbff;
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  .settings-mode-panel > div {
    flex: 1 1 320px;
    min-width: 0;
  }

  .settings-mode-panel h3 {
    margin: 0;
    color: var(--text-main);
    font-size: 16px;
    font-weight: 800;
  }

  .settings-mode-panel p {
    margin: 4px 0 0;
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.4;
  }

  .settings-mode-panel button {
    flex: 0 1 auto;
    max-width: 100%;
    min-height: 34px;
    padding: 0 12px;
    background: #ffffff;
    border: 1px solid #b8d4f4;
    border-radius: 8px;
    color: #1f66b1;
    font-weight: 800;
    white-space: normal;
    cursor: pointer;
  }

  @media (max-width: 640px) {
    .settings-mode-panel {
      align-items: stretch;
    }

    .settings-mode-panel > div,
    .settings-mode-panel button {
      flex-basis: 100%;
    }
  }
</style>
