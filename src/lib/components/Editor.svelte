<script lang="ts">
  import { editorStore } from "../stores/editor.svelte";

  let textareaRef: HTMLTextAreaElement | undefined = $state();
  let lastPath: string | null = $state(null);

  // Auto-focus textarea when a note is loaded
  $effect(() => {
    if (editorStore.path && editorStore.path !== lastPath) {
      lastPath = editorStore.path;
      // Small delay to ensure DOM is ready
      requestAnimationFrame(() => {
        textareaRef?.focus();
      });
    }
  });

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
    // Force save on Ctrl+S - explicitly check for 's' key
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
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
      tabindex="0"
    ></textarea>
  {:else}
    <div class="empty-state">
      <div class="empty-icon">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
          <polyline points="14,2 14,8 20,8" />
          <line x1="16" y1="13" x2="8" y2="13" />
          <line x1="16" y1="17" x2="8" y2="17" />
          <polyline points="10,9 9,9 8,9" />
        </svg>
      </div>
      <p class="empty-title">No note selected</p>
      <p class="empty-hint">Select a note from the list or create a new one</p>

      <div class="quick-actions">
        <div class="action-item">
          <div class="action-keys">
            <kbd>Ctrl</kbd><span class="plus">+</span><kbd>N</kbd>
          </div>
          <span class="action-label">New note</span>
        </div>
        <div class="action-item">
          <div class="action-keys">
            <kbd>Ctrl</kbd><span class="plus">+</span><kbd>R</kbd>
          </div>
          <span class="action-label">Start recording</span>
        </div>
        <div class="action-item">
          <div class="action-keys">
            <kbd>Ctrl</kbd><span class="plus">+</span><kbd>/</kbd>
          </div>
          <span class="action-label">All shortcuts</span>
        </div>
      </div>
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

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    padding: var(--space-xl);
  }

  .empty-icon {
    color: var(--text-disabled);
    opacity: 0.5;
    margin-bottom: var(--space-sm);
  }

  .empty-title {
    font-size: var(--font-size-base);
    font-weight: 500;
    color: var(--text-secondary);
  }

  .empty-hint {
    font-size: var(--font-size-sm);
    color: var(--text-disabled);
    margin-bottom: var(--space-lg);
  }

  .quick-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    padding: var(--space-md);
    background: var(--surface-1);
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
  }

  .action-item {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }

  .action-keys {
    display: flex;
    align-items: center;
    gap: 3px;
    min-width: 80px;
  }

  .action-keys kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 22px;
    height: 22px;
    padding: 0 6px;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 500;
    color: var(--text-primary);
    background: var(--surface-2);
    border: 1px solid var(--border-default);
    border-radius: 4px;
    box-shadow: 0 1px 0 var(--border-strong);
  }

  .plus {
    font-size: 10px;
    color: var(--text-disabled);
  }

  .action-label {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }
</style>
