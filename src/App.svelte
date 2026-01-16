<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Toolbar from "./lib/components/Toolbar.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import NoteList from "./lib/components/NoteList.svelte";
  import Editor from "./lib/components/Editor.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import KeyboardShortcuts from "./lib/components/KeyboardShortcuts.svelte";
  import Settings from "./lib/components/Settings.svelte";
  import LockScreen from "./lib/components/LockScreen.svelte";
  import SetupWizard from "./lib/components/SetupWizard.svelte";
  import { uiStore } from "./lib/stores/ui.svelte";
  import { notesStore } from "./lib/stores/notes.svelte";
  import { editorStore } from "./lib/stores/editor.svelte";
  import { recordingStore } from "./lib/stores/recording.svelte";
  import { themeStore } from "./lib/stores/theme.svelte";
  import { vaultStore } from "./lib/stores/vault.svelte";

  let showShortcuts = $state(false);
  let vaultReady = $state(false);

  // Throttled activity tracking to reset auto-lock timer
  let lastActivityTime = 0;
  function trackActivity() {
    const now = Date.now();
    // Throttle to max once per second to avoid excessive IPC
    if (now - lastActivityTime > 1000) {
      lastActivityTime = now;
      invoke('vault_activity').catch(() => {});
    }
  }

  onMount(async () => {
    themeStore.init();
    await vaultStore.checkStatus();
    vaultReady = true;
  });

  async function handleKeydown(event: KeyboardEvent) {
    // Track activity to reset auto-lock timer
    trackActivity();

    // Close shortcuts on Escape
    if (event.key === "Escape" && showShortcuts) {
      showShortcuts = false;
      return;
    }

    // Delete selected note/folder (only when not in an input/textarea)
    if (event.key === "Delete" && !isInputFocused()) {
      event.preventDefault();
      handleDelete();
      return;
    }

    // Panel-based keyboard navigation (only when not in an input/textarea)
    if (!isInputFocused()) {
      const panel = uiStore.focusedPanel;

      // Up/Down: navigate within current panel
      if (event.key === "ArrowDown" || event.key === "ArrowUp") {
        event.preventDefault();
        if (panel === 'folders' || (panel === 'notes' && uiStore.sidebarVisible && notesStore.notes.length === 0)) {
          // Navigate folders and preview first note
          uiStore.setFocusedPanel('folders');
          if (event.key === "ArrowDown") {
            await notesStore.selectNextFolder();
          } else {
            await notesStore.selectPreviousFolder();
          }
          // Load first note as preview
          if (notesStore.notes.length > 0) {
            notesStore.selectNote(notesStore.notes[0].id);
            editorStore.loadNote(notesStore.notes[0].path);
          }
        } else {
          // Navigate notes and auto-load into editor
          uiStore.setFocusedPanel('notes');
          if (notesStore.notes.length > 0) {
            // Check if current selection exists in this folder's notes
            const noteExists = notesStore.notes.some(n => n.id === notesStore.selectedNoteId);
            if (!noteExists) {
              // Select first note if no valid selection
              notesStore.selectNote(notesStore.notes[0].id);
            } else if (event.key === "ArrowDown") {
              notesStore.selectNextNote();
            } else {
              notesStore.selectPreviousNote();
            }
            // Auto-load selected note into editor
            const path = notesStore.getSelectedNotePath();
            if (path) {
              editorStore.loadNote(path);
            }
          }
        }
        return;
      }

      // Right/Enter: move to next panel or enter editor
      if (event.key === "ArrowRight" || event.key === "Enter") {
        event.preventDefault();
        if (panel === 'folders') {
          // Move to notes panel and ensure a valid note is selected
          uiStore.setFocusedPanel('notes');
          if (notesStore.notes.length > 0) {
            // Check if current selection exists in this folder's notes
            const noteExists = notesStore.notes.some(n => n.id === notesStore.selectedNoteId);
            if (!noteExists) {
              notesStore.selectNote(notesStore.notes[0].id);
            }
            // Load the note into editor (preview only, no focus)
            const path = notesStore.getSelectedNotePath();
            if (path) {
              editorStore.loadNote(path);
            }
          }
        } else if (panel === 'notes') {
          // Load selected note and move to editor
          const path = notesStore.getSelectedNotePath();
          if (path) {
            editorStore.loadNote(path);
            uiStore.setFocusedPanel('editor');
            // Focus the editor textarea
            setTimeout(() => {
              const editor = document.querySelector('.editor-textarea') as HTMLTextAreaElement;
              editor?.focus();
            }, 0);
          }
        }
        return;
      }

      // Left: move to previous panel
      if (event.key === "ArrowLeft") {
        event.preventDefault();
        if (panel === 'editor') {
          uiStore.setFocusedPanel('notes');
        } else if (panel === 'notes' && uiStore.sidebarVisible) {
          uiStore.setFocusedPanel('folders');
        }
        return;
      }
    }

    // Esc in editor: return to notes panel (works even when input is focused)
    if (event.key === "Escape" && uiStore.focusedPanel === 'editor') {
      event.preventDefault();
      // Blur the textarea
      const editor = document.querySelector('.editor-textarea') as HTMLTextAreaElement;
      editor?.blur();
      uiStore.setFocusedPanel('notes');
      return;
    }

    if (event.ctrlKey || event.metaKey) {
      switch (event.key.toLowerCase()) {
        case "b":
          event.preventDefault();
          uiStore.toggleSidebar();
          break;
        case "l":
          event.preventDefault();
          uiStore.toggleNoteList();
          break;
        case "n":
          event.preventDefault();
          handleNewNote();
          break;
        case "r":
          if (!event.shiftKey) {
            event.preventDefault();
            handleToggleRecording();
          }
          break;
        case "s":
          // Prevent browser save dialog - actual save is handled by Editor
          event.preventDefault();
          break;
        case "/":
          event.preventDefault();
          showShortcuts = !showShortcuts;
          break;
        case "d":
          event.preventDefault();
          handleDelete();
          break;
      }
    }
  }

  function isInputFocused(): boolean {
    const active = document.activeElement;
    return active instanceof HTMLInputElement ||
           active instanceof HTMLTextAreaElement ||
           active?.getAttribute("contenteditable") === "true";
  }

  async function handleDelete() {
    const panel = uiStore.focusedPanel;

    if (panel === 'folders') {
      // Delete selected folder
      const folder = notesStore.selectedFolder;
      if (folder && folder !== 'inbox') {
        const folderInfo = notesStore.folders.find(f => f.path === folder)
          || notesStore.folders.flatMap(f => f.children).find(c => c.path === folder);
        const name = folderInfo?.name || folder;
        if (confirm(`Delete folder "${name}" and all its notes?`)) {
          await notesStore.removeFolder(folder);
        }
      }
    } else {
      // Delete selected note (notes or editor panel)
      const notePath = notesStore.getSelectedNotePath();
      if (notePath) {
        if (confirm("Delete this note?")) {
          await notesStore.removeNote(notePath);
          if (notesStore.notes.length > 0) {
            editorStore.loadNote(notesStore.notes[0].path);
          } else {
            editorStore.clear();
          }
        }
      }
    }
  }

  async function handleDeleteSelectedNote() {
    if (!editorStore.path) return;

    if (confirm("Delete this note?")) {
      const pathToDelete = editorStore.path;
      await notesStore.removeNote(pathToDelete);
      // Load first available note or clear editor
      if (notesStore.notes.length > 0) {
        editorStore.loadNote(notesStore.notes[0].path);
      } else {
        editorStore.clear();
      }
    }
  }

  async function handleNewNote() {
    const path = await notesStore.addNote();
    if (path) {
      editorStore.loadNote(path);
    }
  }

  async function handleToggleRecording() {
    if (recordingStore.status === "idle") {
      recordingStore.startRecording();
    } else if (recordingStore.status === "recording") {
      const transcription = await recordingStore.stopRecording();
      if (transcription) {
        // Auto-create a new note if none is selected
        if (!editorStore.path) {
          const newPath = await notesStore.addNote();
          if (newPath) {
            await editorStore.loadNote(newPath);
          }
        }
        editorStore.insertAtCursor(transcription);
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} onclick={trackActivity} />

{#if !vaultReady}
  <!-- Loading state while checking vault -->
  <div class="loading">
    <div class="spinner"></div>
  </div>
{:else if !vaultStore.status.initialized}
  <!-- First run - show setup wizard -->
  <SetupWizard />
{:else if vaultStore.status.locked}
  <!-- Vault locked - show lock screen -->
  <LockScreen />
{:else}
  <!-- Vault unlocked - show main app -->
  <div class="app">
    <Toolbar onshowshortcuts={() => (showShortcuts = true)} />

    <div class="main">
      <aside class="sidebar" class:collapsed={!uiStore.sidebarVisible} class:pane-focused={uiStore.focusedPanel === 'folders'}>
        <Sidebar />
      </aside>

      <section class="note-list" class:collapsed={!uiStore.noteListVisible} class:pane-focused={uiStore.focusedPanel === 'notes'}>
        <NoteList />
      </section>

      <main class="editor" class:pane-focused={uiStore.focusedPanel === 'editor'}>
        <Editor />
      </main>
    </div>

    <StatusBar />
  </div>

  <KeyboardShortcuts visible={showShortcuts} onclose={() => (showShortcuts = false)} />
  <Settings visible={uiStore.settingsOpen} onclose={() => uiStore.closeSettings()} />
{/if}

<style>
  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--bg-primary);
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface-0);
  }

  .main {
    display: flex;
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .sidebar {
    width: var(--sidebar-width);
    height: 100%;
    background: var(--surface-1);
    border-right: 1px solid var(--divider);
    overflow-y: auto;
    flex-shrink: 0;
    transition:
      width var(--transition-slow),
      opacity var(--transition-slow),
      margin var(--transition-slow);
  }

  .sidebar.collapsed {
    width: 0;
    opacity: 0;
    overflow: hidden;
    border-right: none;
  }

  .note-list {
    width: var(--notelist-width);
    height: 100%;
    background: var(--surface-1);
    border-right: 1px solid var(--divider);
    overflow-y: auto;
    flex-shrink: 0;
    transition:
      width var(--transition-slow),
      opacity var(--transition-slow),
      margin var(--transition-slow);
  }

  .note-list.collapsed {
    width: 0;
    opacity: 0;
    overflow: hidden;
    border-right: none;
  }

  .editor {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  /* Pane focus indicator - subtle top border */
  .sidebar.pane-focused,
  .note-list.pane-focused,
  .editor.pane-focused {
    border-top: 2px solid var(--accent);
  }

  .sidebar:not(.pane-focused),
  .note-list:not(.pane-focused),
  .editor:not(.pane-focused) {
    border-top: 2px solid transparent;
  }

  </style>
