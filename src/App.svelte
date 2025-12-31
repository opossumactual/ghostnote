<script lang="ts">
  import Toolbar from "./lib/components/Toolbar.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import NoteList from "./lib/components/NoteList.svelte";
  import Editor from "./lib/components/Editor.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import { uiStore } from "./lib/stores/ui.svelte";
  import { notesStore } from "./lib/stores/notes.svelte";
  import { editorStore } from "./lib/stores/editor.svelte";
  import { recordingStore } from "./lib/stores/recording.svelte";

  function handleKeydown(event: KeyboardEvent) {
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
        editorStore.insertAtCursor(transcription);
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app">
  <Toolbar />

  <div class="main">
    {#if uiStore.sidebarVisible}
      <aside class="sidebar">
        <Sidebar />
      </aside>
    {/if}

    {#if uiStore.noteListVisible}
      <section class="note-list">
        <NoteList />
      </section>
    {/if}

    <main class="editor">
      <Editor />
    </main>
  </div>

  <StatusBar />
</div>

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
  }

  .sidebar {
    width: var(--sidebar-width);
    background: var(--surface-1);
    border-right: 1px solid var(--divider);
    overflow-y: auto;
    flex-shrink: 0;
  }

  .note-list {
    width: var(--notelist-width);
    background: var(--surface-1);
    border-right: 1px solid var(--divider);
    overflow-y: auto;
    flex-shrink: 0;
  }

  .editor {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
