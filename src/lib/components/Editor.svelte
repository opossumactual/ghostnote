<script lang="ts">
  import { editorStore } from "../stores/editor.svelte";

  let textareaRef: HTMLTextAreaElement | undefined = $state();

  function handleInput(event: Event) {
    const target = event.target as HTMLTextAreaElement;
    editorStore.updateContent(target.value);
  }

  function handleSelect() {
    if (textareaRef) {
      editorStore.updateCursor(textareaRef.selectionStart);
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    // Force save on Ctrl+S
    if ((event.ctrlKey || event.metaKey) && event.key === "s") {
      event.preventDefault();
      editorStore.save();
    }
  }
</script>

<div class="editor-container">
  {#if editorStore.path}
    <textarea
      bind:this={textareaRef}
      class="editor-textarea"
      value={editorStore.content}
      oninput={handleInput}
      onselect={handleSelect}
      onclick={handleSelect}
      onkeyup={handleSelect}
      onkeydown={handleKeydown}
      placeholder="Start typing or use voice transcription..."
      spellcheck="true"
    ></textarea>
  {:else}
    <div class="no-note">
      <p>Select a note or create a new one</p>
    </div>
  {/if}
</div>

<style>
  .editor-container {
    flex: 1;
    padding: var(--space-lg);
    overflow: hidden;
    display: flex;
  }

  .editor-textarea {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--font-size-base);
    line-height: 1.7;
    color: var(--text-primary);
    background: transparent;
    resize: none;
    padding: 0;
  }

  .editor-textarea:focus {
    outline: none;
  }

  .editor-textarea::placeholder {
    color: var(--text-disabled);
  }

  .no-note {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-disabled);
  }
</style>
