<script lang="ts">
  import { recordingStore } from "../stores/recording.svelte";
  import { editorStore } from "../stores/editor.svelte";

  async function toggleRecording() {
    if (recordingStore.status === "idle") {
      recordingStore.startRecording();
    } else if (recordingStore.status === "recording") {
      const transcription = await recordingStore.stopRecording();
      if (transcription) {
        editorStore.insertAtCursor(transcription);
      }
    }
  }

  const statusLabel = $derived(
    recordingStore.status === "idle"
      ? "Record"
      : recordingStore.status === "recording"
        ? recordingStore.formattedDuration
        : "Processing..."
  );
</script>

<button
  class="record-btn"
  class:recording={recordingStore.isRecording}
  class:processing={recordingStore.isProcessing}
  onclick={toggleRecording}
  disabled={recordingStore.isProcessing}
  title="Toggle recording (Ctrl+R)"
>
  {#if recordingStore.isProcessing}
    <svg class="spinner" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10" stroke-dasharray="32" stroke-dashoffset="32" />
    </svg>
  {:else}
    <svg width="18" height="18" viewBox="0 0 24 24" fill={recordingStore.isRecording ? "currentColor" : "none"} stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="6" />
    </svg>
  {/if}
  <span>{statusLabel}</span>
</button>

<style>
  .record-btn {
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
    position: relative;
    min-width: 90px;
    flex-shrink: 0;
  }

  .record-btn:hover:not(:disabled) {
    background: var(--surface-2);
    color: var(--text-primary);
    border-color: var(--text-ghost);
    text-shadow: 0 0 10px var(--accent-glow);
  }

  .record-btn:disabled {
    opacity: 0.7;
    cursor: wait;
  }

  .record-btn span {
    font-size: var(--font-size-xs);
  }

  .record-btn.recording {
    color: var(--recording);
    border-color: var(--recording);
    background: var(--recording-dim);
    box-shadow: 0 0 20px var(--recording-glow);
    animation: pulse 1s ease-in-out infinite;
  }

  .record-btn.recording::before {
    content: 'REC';
    position: absolute;
    top: -6px;
    right: -6px;
    font-size: 8px;
    padding: 1px 4px;
    background: var(--recording);
    color: var(--surface-0);
    animation: blink 0.5s step-end infinite;
  }

  .record-btn.processing {
    color: var(--accent);
  }

  .spinner {
    animation: spin 1s linear infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      box-shadow: 0 0 10px var(--recording-glow);
    }
    50% {
      box-shadow: 0 0 25px var(--recording-glow);
    }
  }

  @keyframes blink {
    50% { opacity: 0; }
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
