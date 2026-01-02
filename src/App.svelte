<script lang="ts">
  import { onMount } from "svelte";
  import Toolbar from "./lib/components/Toolbar.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import NoteList from "./lib/components/NoteList.svelte";
  import Editor from "./lib/components/Editor.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import KeyboardShortcuts from "./lib/components/KeyboardShortcuts.svelte";
  import Settings from "./lib/components/Settings.svelte";
  import { uiStore } from "./lib/stores/ui.svelte";
  import { notesStore } from "./lib/stores/notes.svelte";
  import { editorStore } from "./lib/stores/editor.svelte";
  import { recordingStore } from "./lib/stores/recording.svelte";
  import { themeStore } from "./lib/stores/theme.svelte";

  let showShortcuts = $state(false);

  onMount(() => {
    themeStore.init();
  });

  function handleKeydown(event: KeyboardEvent) {
    // Close shortcuts on Escape
    if (event.key === "Escape" && showShortcuts) {
      showShortcuts = false;
      return;
    }

    // Delete selected note (only when not in an input/textarea)
    if (event.key === "Delete" && !isInputFocused()) {
      event.preventDefault();
      handleDeleteSelectedNote();
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
      }
    }
  }

  function isInputFocused(): boolean {
    const active = document.activeElement;
    return active instanceof HTMLInputElement ||
           active instanceof HTMLTextAreaElement ||
           active?.getAttribute("contenteditable") === "true";
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

<svelte:window onkeydown={handleKeydown} />

<div class="app">
  <Toolbar onshowshortcuts={() => (showShortcuts = true)} />

  <div class="main">
    <aside class="sidebar" class:collapsed={!uiStore.sidebarVisible}>
      <Sidebar />
    </aside>

    <section class="note-list" class:collapsed={!uiStore.noteListVisible}>
      <NoteList />
    </section>

    <main class="editor">
      <Editor />
    </main>
  </div>

  <StatusBar />
</div>

<KeyboardShortcuts visible={showShortcuts} onclose={() => (showShortcuts = false)} />
<Settings visible={uiStore.settingsOpen} onclose={() => uiStore.closeSettings()} />

<style>
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
</style>
