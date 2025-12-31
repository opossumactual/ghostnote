<script lang="ts">
  import { notesStore } from "../stores/notes.svelte";
  import { editorStore } from "../stores/editor.svelte";

  function selectNote(id: string, path: string) {
    notesStore.selectNote(id);
    editorStore.loadNote(path);
  }
</script>

<div class="note-list-content">
  <div class="list-header">
    <span class="note-count">{notesStore.notes.length} notes</span>
  </div>

  <div class="notes">
    {#if notesStore.isLoading}
      <div class="loading">Loading...</div>
    {:else if notesStore.notes.length === 0}
      <div class="empty">No notes yet</div>
    {:else}
      {#each notesStore.notes as note}
        <button
          class="note-item"
          class:selected={notesStore.selectedNoteId === note.id}
          onclick={() => selectNote(note.id, note.path)}
        >
          <div class="note-title">{note.title}</div>
          <div class="note-meta">
            <span class="note-date">{note.modified}</span>
            <span class="note-words">{note.word_count}w</span>
          </div>
          <div class="note-preview">{note.preview}</div>
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .note-list-content {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .list-header {
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border-subtle);
  }

  .note-count {
    font-size: var(--font-size-xs);
    color: var(--text-disabled);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .notes {
    flex: 1;
    overflow-y: auto;
  }

  .loading,
  .empty {
    padding: var(--space-lg);
    text-align: center;
    color: var(--text-disabled);
    font-size: var(--font-size-sm);
  }

  .note-item {
    display: block;
    width: 100%;
    padding: var(--space-md);
    text-align: left;
    border-bottom: 1px solid var(--border-subtle);
    transition: background var(--transition-fast);
  }

  .note-item:hover {
    background: var(--surface-2);
  }

  .note-item.selected {
    background: var(--surface-2);
    border-left: 3px solid var(--accent);
    padding-left: calc(var(--space-md) - 3px);
  }

  .note-title {
    font-size: var(--font-size-sm);
    font-weight: 500;
    color: var(--text-primary);
    margin-bottom: var(--space-xs);
  }

  .note-meta {
    display: flex;
    gap: var(--space-sm);
    font-size: var(--font-size-xs);
    color: var(--text-disabled);
    margin-bottom: var(--space-xs);
  }

  .note-preview {
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
