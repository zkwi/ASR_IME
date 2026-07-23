<script lang="ts">
  import { Check, Copy, Eye, EyeOff, X } from "lucide-svelte";

  type Props = {
    id: string;
    configField: string;
    value: string;
    label: string;
    hint?: string;
    error?: string;
    showLabel: string;
    hideLabel: string;
    copyLabel: string;
    copiedLabel: string;
    copyFailedLabel: string;
  };

  let {
    id,
    configField,
    value = $bindable(""),
    label,
    hint = "",
    error = "",
    showLabel,
    hideLabel,
    copyLabel,
    copiedLabel,
    copyFailedLabel,
  }: Props = $props();

  let revealed = $state(false);
  let copyState = $state<"idle" | "copied" | "failed">("idle");
  let hintId = $derived(hint ? `${id}-hint` : undefined);
  let errorId = $derived(error ? `${id}-error` : undefined);
  let describedBy = $derived([errorId, hintId].filter(Boolean).join(" ") || undefined);
  let copyStatusLabel = $derived(
    copyState === "copied" ? copiedLabel : copyState === "failed" ? copyFailedLabel : copyLabel,
  );

  async function copySecret() {
    if (!value) return;
    try {
      await navigator.clipboard.writeText(value);
      copyState = "copied";
    } catch {
      copyState = "failed";
    }
  }
</script>

<div class="secret-field" class:field-invalid={Boolean(error)}>
  <label for={id}>{label}</label>
  <span class="secret-input-shell">
    <input
      {id}
      data-config-field={configField}
      aria-invalid={Boolean(error)}
      aria-describedby={describedBy}
      type={revealed ? "text" : "password"}
      autocomplete="off"
      bind:value
      oninput={() => (copyState = "idle")}
    />
    <span class="secret-input-actions">
      <button
        type="button"
        class="secret-action"
        aria-label={revealed ? hideLabel : showLabel}
        title={revealed ? hideLabel : showLabel}
        aria-pressed={revealed}
        onclick={() => (revealed = !revealed)}
      >
        {#if revealed}<EyeOff size={17} />{:else}<Eye size={17} />{/if}
      </button>
      <button
        type="button"
        class:copied={copyState === "copied"}
        class:failed={copyState === "failed"}
        class="secret-action"
        aria-label={copyStatusLabel}
        title={copyStatusLabel}
        disabled={!value}
        onclick={copySecret}
      >
        {#if copyState === "copied"}
          <Check size={17} />
        {:else if copyState === "failed"}
          <X size={17} />
        {:else}
          <Copy size={17} />
        {/if}
      </button>
    </span>
  </span>
  <span class="sr-only" aria-live="polite">{copyState === "idle" ? "" : copyStatusLabel}</span>
  {#if error}<small id={errorId} class="field-error">{error}</small>{/if}
  {#if hint}<small id={hintId} class="field-hint">{hint}</small>{/if}
</div>

<style>
  .secret-field {
    display: grid;
    align-content: start;
    gap: 8px;
    min-width: 0;
    color: var(--text-secondary);
    font-size: 14px;
  }

  .secret-input-shell {
    position: relative;
    display: block;
    min-width: 0;
  }

  .secret-input-shell input {
    padding-right: 82px !important;
  }

  .secret-input-actions {
    position: absolute;
    top: 50%;
    right: 6px;
    display: inline-flex;
    gap: 2px;
    transform: translateY(-50%);
  }

  .secret-action {
    display: grid;
    width: 32px;
    height: 30px;
    place-items: center;
    padding: 0;
    color: var(--text-secondary);
    background: transparent;
    border: 0;
    border-radius: 8px;
  }

  .secret-action:hover:not(:disabled) {
    color: var(--primary);
    background: var(--primary-light);
  }

  .secret-action.copied {
    color: #07885d;
    background: #eafaf4;
  }

  .secret-action.failed {
    color: #c24141;
    background: #fff1f1;
  }

  .secret-action:disabled {
    cursor: not-allowed;
    opacity: 0.35;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
