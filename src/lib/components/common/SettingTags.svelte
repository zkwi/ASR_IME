<script lang="ts">
  export type SettingTag = string | { label: string; tone?: "default" | "required" };

  type Props = {
    tags: SettingTag[];
  };

  let { tags }: Props = $props();

  function labelOf(tag: SettingTag) {
    return typeof tag === "string" ? tag : tag.label;
  }

  function isRequired(tag: SettingTag) {
    return typeof tag !== "string" && tag.tone === "required";
  }
</script>

{#if tags.length > 0}
  <div class="setting-tags">
    {#each tags as tag}
      <span class:required={isRequired(tag)} class="setting-tag">{labelOf(tag)}</span>
    {/each}
  </div>
{/if}

<style>
  .setting-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    min-width: 0;
    margin-top: 8px;
  }

  .setting-tag {
    display: inline-flex;
    align-items: center;
    min-height: 22px;
    padding: 0 8px;
    color: #50627a;
    background: #f1f6fd;
    border: 1px solid #dce8f6;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 800;
    line-height: 1.2;
    overflow-wrap: anywhere;
    text-align: center;
    white-space: normal;
  }

  .setting-tag.required {
    color: #1f66b1;
    background: #eaf3ff;
    border-color: #c7def8;
  }
</style>
