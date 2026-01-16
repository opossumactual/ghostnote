<script lang="ts">
  interface Props {
    visible: boolean;
    onclose: () => void;
  }

  let { visible, onclose }: Props = $props();

  const shortcuts = [
    { keys: ["Ctrl", "B"], action: "Toggle sidebar" },
    { keys: ["Ctrl", "L"], action: "Toggle note list" },
    { keys: ["Ctrl", "N"], action: "Create new note" },
    { keys: ["Ctrl", "R"], action: "Toggle recording" },
    { keys: ["Ctrl", "S"], action: "Save note" },
    { keys: ["Ctrl", "/"], action: "Show shortcuts" },
    { keys: ["↑", "/", "↓"], action: "Navigate in panel" },
    { keys: ["←", "/", "→"], action: "Switch panels" },
    { keys: ["Enter"], action: "Enter panel / Open note" },
    { keys: ["Ctrl", "D"], action: "Delete note/folder" },
    { keys: ["Esc"], action: "Close dialogs" },
  ];

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onclose();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onclose();
    }
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div
    class="overlay"
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
    role="dialog"
    aria-modal="true"
    aria-labelledby="shortcuts-title"
    tabindex="0"
  >
    <div class="shortcuts-panel">
      <header class="panel-header">
        <h2 id="shortcuts-title">Keyboard Shortcuts</h2>
        <button class="close-btn" onclick={onclose} aria-label="Close">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </header>

      <div class="shortcuts-list">
        {#each shortcuts as shortcut}
          <div class="shortcut-row">
            <div class="keys">
              {#each shortcut.keys as key, i}
                <kbd>{key}</kbd>
                {#if i < shortcut.keys.length - 1}
                  <span class="plus">+</span>
                {/if}
              {/each}
            </div>
            <span class="action">{shortcut.action}</span>
          </div>
        {/each}
      </div>

      <footer class="panel-footer">
        <span class="hint">Press <kbd>Esc</kbd> to close</span>
      </footer>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeIn 150ms ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .shortcuts-panel {
    background: var(--surface-2);
    border: 1px solid var(--border-default);
    border-radius: 12px;
    width: 340px;
    max-width: 90vw;
    box-shadow:
      0 24px 48px rgba(0, 0, 0, 0.4),
      0 0 0 1px rgba(255, 255, 255, 0.05) inset;
    animation: slideUp 200ms ease-out;
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(16px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--border-subtle);
  }

  .panel-header h2 {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.01em;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 6px;
    color: var(--text-secondary);
    transition: all var(--transition-fast);
  }

  .close-btn:hover {
    background: var(--surface-3);
    color: var(--text-primary);
  }

  .shortcuts-list {
    padding: var(--space-md) var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-xs) 0;
  }

  .keys {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 24px;
    padding: 0 8px;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 500;
    color: var(--text-primary);
    background: var(--surface-0);
    border: 1px solid var(--border-default);
    border-radius: 5px;
    box-shadow:
      0 1px 0 var(--border-strong),
      0 2px 0 var(--surface-0);
  }

  .plus {
    font-size: 10px;
    color: var(--text-disabled);
    margin: 0 2px;
  }

  .action {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  .panel-footer {
    padding: var(--space-sm) var(--space-lg);
    border-top: 1px solid var(--border-subtle);
    text-align: center;
  }

  .hint {
    font-size: var(--font-size-xs);
    color: var(--text-disabled);
  }

  .hint kbd {
    font-size: 10px;
    height: 18px;
    min-width: 18px;
    padding: 0 5px;
  }
</style>
