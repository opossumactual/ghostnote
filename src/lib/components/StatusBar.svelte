<script lang="ts">
  import { editorStore } from "../stores/editor.svelte";

  const saveLabel = $derived(
    editorStore.isSaving
      ? "Saving..."
      : editorStore.isDirty
        ? "Unsaved"
        : "Saved"
  );
</script>

<footer class="status-bar">
  <div class="status-left">
    <span class="word-count">{editorStore.wordCount} words</span>
  </div>

  <div class="status-right">
    <span class="save-status" class:unsaved={editorStore.isDirty}>
      {saveLabel}
    </span>
  </div>
</footer>

<style>
  .status-bar {
    height: var(--statusbar-height);
    background: var(--surface-1);
    border-top: 1px solid var(--divider);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-md);
    font-size: var(--font-size-xs);
    color: var(--text-disabled);
    flex-shrink: 0;
  }

  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }

  .save-status.unsaved {
    color: var(--warning);
  }
</style>
