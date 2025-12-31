<script lang="ts">
  import RecordButton from "./RecordButton.svelte";
  import { notesStore } from "../stores/notes.svelte";
  import { editorStore } from "../stores/editor.svelte";
  import { uiStore } from "../stores/ui.svelte";

  async function handleNewNote() {
    const path = await notesStore.addNote();
    if (path) {
      editorStore.loadNote(path);
    }
  }

  function handleSettings() {
    uiStore.openSettings();
  }
</script>

<header class="toolbar">
  <div class="toolbar-left">
    <button class="toolbar-btn" onclick={handleNewNote} title="New note (Ctrl+N)">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 5v14M5 12h14" />
      </svg>
      <span>New</span>
    </button>

    <RecordButton />
  </div>

  <div class="toolbar-center">
    <span class="app-title">opnotes</span>
  </div>

  <div class="toolbar-right">
    <button class="toolbar-btn" onclick={handleSettings} title="Settings">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3" />
        <path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4M4.22 19.78l2.83-2.83M16.95 7.05l2.83-2.83" />
      </svg>
    </button>
  </div>
</header>

<style>
  .toolbar {
    height: var(--toolbar-height);
    background: var(--surface-1);
    border-bottom: 1px solid var(--divider);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-md);
    flex-shrink: 0;
  }

  .toolbar-left,
  .toolbar-right {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .toolbar-center {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
  }

  .app-title {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    font-weight: 500;
    user-select: none;
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-sm);
    border-radius: 6px;
    color: var(--text-secondary);
    transition: all var(--transition-fast);
  }

  .toolbar-btn:hover {
    background: var(--surface-2);
    color: var(--text-primary);
  }

  .toolbar-btn span {
    font-size: var(--font-size-sm);
  }
</style>
