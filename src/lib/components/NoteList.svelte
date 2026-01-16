<script lang="ts">
  import { notesStore } from "../stores/notes.svelte";
  import { editorStore } from "../stores/editor.svelte";
  import { uiStore } from "../stores/ui.svelte";

  function selectNote(id: string, path: string) {
    notesStore.selectNote(id);
    editorStore.loadNote(path);
    uiStore.setFocusedPanel('notes');
  }

  async function handleDelete(event: MouseEvent, path: string) {
    event.stopPropagation();
    if (confirm("Delete this note?")) {
      await notesStore.removeNote(path);
      // Clear editor if deleted note was selected
      if (editorStore.path === path) {
        // Reset editor state by loading nothing
        window.location.reload(); // Simple approach for now
      }
    }
  }
</script>

<div class="note-list-content">
  <div class="list-header">
    <span class="note-count">{notesStore.notes.length} notes</span>
  </div>

  <div class="notes">
    {#if notesStore.isLoading}
      <div class="loading">
        <div class="loading-spinner"></div>
        <span>Loading notes...</span>
      </div>
    {:else if notesStore.notes.length === 0}
      <div class="empty">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
          <polyline points="14,2 14,8 20,8" />
        </svg>
        <p class="empty-title">No notes yet</p>
        <p class="empty-hint">Press <kbd>Ctrl</kbd>+<kbd>N</kbd> to create one</p>
      </div>
    {:else}
      {#each notesStore.notes as note}
        <div
          class="note-item"
          class:selected={notesStore.selectedNoteId === note.id}
          onclick={() => selectNote(note.id, note.path)}
        >
          <div class="note-content">
            <div class="note-title">{note.title}</div>
            <div class="note-meta">
              <span class="note-date">{note.modified}</span>
              <span class="note-words">{note.word_count}w</span>
            </div>
            <div class="note-preview">{note.preview}</div>
          </div>
          <button
            class="delete-btn"
            onclick={(e) => handleDelete(e, note.path)}
            title="Delete note"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" />
            </svg>
          </button>
        </div>
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
    border-bottom: 1px dashed var(--text-ghost);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .note-count {
    font-family: var(--font-display), monospace;
    font-size: 14px;
    color: var(--text-ghost);
    letter-spacing: 1px;
  }

  .notes {
    flex: 1;
    overflow-y: auto;
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xl);
    color: var(--text-disabled);
    font-size: var(--font-size-sm);
  }

  .loading-spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--surface-3);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xl) var(--space-lg);
    text-align: center;
    color: var(--text-disabled);
  }

  .empty svg {
    opacity: 0.4;
    margin-bottom: var(--space-xs);
  }

  .empty-title {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  .empty-hint {
    font-size: var(--font-size-xs);
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .empty-hint kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    color: var(--text-secondary);
    background: var(--surface-2);
    border: 1px solid var(--border-subtle);
    border-radius: 3px;
  }

  .note-item {
    display: flex;
    align-items: flex-start;
    gap: var(--space-sm);
    width: 100%;
    padding: var(--space-md);
    text-align: left;
    border: 1px solid transparent;
    border-left: 3px solid transparent;
    transition: all var(--transition-fast);
    cursor: pointer;
    margin-bottom: 4px;
    outline: none;
  }

  .note-item:hover:not(.selected) {
    background: var(--surface-3);
    border-color: var(--text-ghost);
    border-left-color: var(--text-ghost);
  }

  .note-item.selected {
    background: var(--accent-dim);
    border-color: var(--accent);
    border-left-color: var(--accent);
  }

  .note-content {
    flex: 1;
    min-width: 0;
  }

  .delete-btn {
    opacity: 0;
    padding: var(--space-xs);
    border-radius: 4px;
    color: var(--text-disabled);
    transition: all var(--transition-fast);
    flex-shrink: 0;
  }

  .note-item:hover .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    background: var(--error-dim);
    color: var(--error);
  }

  .note-title {
    font-size: var(--font-size-sm);
    font-weight: 500;
    color: var(--text-primary);
    margin-bottom: var(--space-xs);
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .note-title::before {
    content: '>';
    color: var(--accent);
  }

  .note-meta {
    display: flex;
    gap: var(--space-sm);
    color: var(--text-ghost);
    margin-bottom: var(--space-xs);
    padding-left: var(--space-md);
    font-family: var(--font-display), monospace;
    font-size: 12px;
  }

  .note-preview {
    font-size: 12px;
    color: var(--text-disabled);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding-left: var(--space-md);
    opacity: 0.85;
  }
</style>
