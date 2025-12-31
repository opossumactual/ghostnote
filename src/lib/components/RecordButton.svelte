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
    border-radius: 6px;
    color: var(--text-secondary);
    transition: all var(--transition-fast);
  }

  .record-btn:hover:not(:disabled) {
    background: var(--surface-2);
    color: var(--text-primary);
  }

  .record-btn:disabled {
    opacity: 0.7;
    cursor: wait;
  }

  .record-btn span {
    font-size: var(--font-size-sm);
  }

  .record-btn.recording {
    color: var(--recording);
    background: rgba(255, 107, 107, 0.1);
    box-shadow: 0 0 20px var(--recording-glow);
    animation: pulse 1.5s ease-in-out infinite;
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

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
