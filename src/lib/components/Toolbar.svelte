<script lang="ts">
  import { getCurrentWindow, type Window } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import RecordButton from "./RecordButton.svelte";
  import { notesStore } from "../stores/notes.svelte";
  import { editorStore } from "../stores/editor.svelte";
  import { uiStore } from "../stores/ui.svelte";

  interface Props {
    onshowshortcuts?: () => void;
  }

  let { onshowshortcuts }: Props = $props();
  let isMaximized = $state(false);
  let appWindow: Window | null = $state(null);

  onMount(() => {
    // Get window reference after mount
    appWindow = getCurrentWindow();

    // Check initial maximized state
    appWindow.isMaximized().then((maximized) => {
      isMaximized = maximized;
    });

    // Listen for window resize to update maximize state
    const unlisten = appWindow.onResized(async () => {
      if (appWindow) {
        isMaximized = await appWindow.isMaximized();
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  });

  async function handleMinimize() {
    if (appWindow) {
      await appWindow.minimize();
    }
  }

  async function handleMaximize() {
    if (appWindow) {
      await appWindow.toggleMaximize();
      isMaximized = await appWindow.isMaximized();
    }
  }

  async function handleClose() {
    if (appWindow) {
      await appWindow.close();
    }
  }

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
    <button
      class="toolbar-btn panel-toggle"
      class:active={uiStore.sidebarVisible}
      onclick={() => uiStore.toggleSidebar()}
      title="Toggle sidebar (Ctrl+B)"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="18" height="18" rx="2" />
        <path d="M9 3v18" />
      </svg>
    </button>

    <button
      class="toolbar-btn panel-toggle"
      class:active={uiStore.noteListVisible}
      onclick={() => uiStore.toggleNoteList()}
      title="Toggle note list (Ctrl+L)"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01" />
      </svg>
    </button>

    <div class="toolbar-divider"></div>

    <button class="toolbar-btn" onclick={handleNewNote} title="New note (Ctrl+N)">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 5v14M5 12h14" />
      </svg>
      <span>New</span>
    </button>

    <RecordButton />
  </div>

  <!-- Drag region behind everything -->
  <div class="toolbar-drag-region" data-tauri-drag-region></div>

  <div class="toolbar-center">
    <span class="app-title">onote</span>
  </div>

  <div class="toolbar-right">
    <button
      class="toolbar-btn help-btn"
      onclick={onshowshortcuts}
      title="Keyboard shortcuts (Ctrl+/)"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10" />
        <path d="M9.09 9a3 3 0 015.83 1c0 2-3 3-3 3" />
        <path d="M12 17h.01" />
      </svg>
    </button>

    <button class="toolbar-btn" onclick={handleSettings} title="Settings">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
        <circle cx="12" cy="12" r="3" />
      </svg>
    </button>

    <div class="toolbar-divider"></div>

    <!-- Window Controls -->
    <div class="window-controls">
      <button class="window-btn" onclick={handleMinimize} title="Minimize" aria-label="Minimize window">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M5 12h14" />
        </svg>
      </button>
      <button class="window-btn" onclick={handleMaximize} title={isMaximized ? "Restore" : "Maximize"} aria-label={isMaximized ? "Restore window" : "Maximize window"}>
        {#if isMaximized}
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="5" y="9" width="10" height="10" rx="1" />
            <path d="M9 9V5a1 1 0 011-1h9a1 1 0 011 1v9a1 1 0 01-1 1h-4" />
          </svg>
        {:else}
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="4" y="4" width="16" height="16" rx="1" />
          </svg>
        {/if}
      </button>
      <button class="window-btn window-btn-close" onclick={handleClose} title="Close" aria-label="Close window">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12" />
        </svg>
      </button>
    </div>
  </div>
</header>

<style>
  .toolbar {
    height: var(--toolbar-height);
    min-height: var(--toolbar-height);
    background: var(--surface-1);
    border-bottom: 1px solid var(--divider);
    display: flex;
    flex-wrap: nowrap;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-md);
    flex-shrink: 0;
    position: relative;
    overflow: hidden;
  }

  .toolbar-drag-region {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 0;
  }

  .toolbar-left,
  .toolbar-right {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    flex-shrink: 0;
    position: relative;
    z-index: 1;
  }

  .toolbar-center {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    z-index: 0;
    pointer-events: none;
  }

  .app-title {
    font-family: var(--font-display), monospace;
    font-size: calc(var(--font-size-xs) * 1.6);
    color: var(--text-disabled);
    font-weight: 400;
    user-select: none;
    letter-spacing: 2px;
    text-transform: uppercase;
  }

  .toolbar-divider {
    width: 1px;
    height: 20px;
    background: var(--border-subtle);
    margin: 0 var(--space-xs);
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-sm);
    border: 1px solid transparent;
    color: var(--text-disabled);
    transition: all var(--transition-fast);
    text-transform: uppercase;
    font-size: var(--font-size-xs);
    letter-spacing: 0.5px;
    flex-shrink: 0;
    white-space: nowrap;
  }

  .toolbar-btn:hover {
    background: var(--surface-2);
    color: var(--text-primary);
    border-color: var(--text-ghost);
    text-shadow: 0 0 10px var(--accent-glow);
  }

  .toolbar-btn span {
    font-size: var(--font-size-xs);
  }

  /* Panel toggle buttons */
  .panel-toggle {
    padding: var(--space-xs);
    position: relative;
  }

  .panel-toggle.active {
    color: var(--accent);
    background: var(--accent-dim);
    border-color: var(--accent);
    box-shadow: 0 0 10px var(--accent-glow);
  }

  .panel-toggle.active:hover {
    background: var(--accent-dim);
    color: var(--accent-hover);
  }


  /* Window Controls */
  .window-controls {
    display: flex;
    align-items: center;
    gap: 2px;
    margin-left: var(--space-xs);
    flex-shrink: 0;
  }

  .window-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    min-width: 28px;
    height: 28px;
    border: 1px solid transparent;
    color: var(--text-disabled);
    transition: all var(--transition-fast);
    flex-shrink: 0;
  }

  .window-btn:hover {
    background: var(--surface-3);
    color: var(--text-primary);
    border-color: var(--text-ghost);
  }

  .window-btn-close:hover {
    background: var(--error);
    color: var(--surface-0);
    border-color: var(--error);
  }
</style>
