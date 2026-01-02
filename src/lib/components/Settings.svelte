<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import * as commands from "../utils/tauri-commands";
  import type { AudioDevice, WhisperModel, ModelStatus, DownloadProgress } from "../utils/tauri-commands";
  import { themeStore } from "../stores/theme.svelte";
  import { vaultStore } from "../stores/vault.svelte";

  interface Props {
    visible: boolean;
    onclose: () => void;
  }

  let { visible, onclose }: Props = $props();

  // Audio devices
  let audioDevices = $state<AudioDevice[]>([]);
  let selectedDevice = $state<string | null>(null);
  let loadingDevices = $state(false);

  // Whisper models
  let whisperModels = $state<WhisperModel[]>([]);
  let modelStatuses = $state<Map<string, ModelStatus>>(new Map());
  let selectedModel = $state("small.en");
  let downloadingModel = $state<string | null>(null);
  let downloadProgress = $state(0);

  // Error state
  let error = $state<string | null>(null);

  // Vault settings
  let autoLockTimeout = $state(5);
  let showChangePassword = $state(false);
  let currentPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let changingPassword = $state(false);
  let passwordError = $state<string | null>(null);

  const autoLockOptions = [
    { value: 1, label: '1 minute' },
    { value: 5, label: '5 minutes' },
    { value: 10, label: '10 minutes' },
    { value: 15, label: '15 minutes' },
    { value: 30, label: '30 minutes' },
  ];

  onMount(async () => {
    // Listen for download progress events
    const unlisten = await listen<DownloadProgress>("model-download-progress", (event) => {
      downloadProgress = event.payload.percent;
    });

    return () => {
      unlisten();
    };
  });

  async function loadSettings() {
    loadingDevices = true;
    error = null;

    try {
      // Load audio devices
      audioDevices = await commands.listAudioDevices();
      selectedDevice = await commands.getSelectedDevice();

      // Load whisper models
      whisperModels = await commands.listWhisperModels();
      selectedModel = await commands.getSelectedModel();

      // Get status for each model
      const statuses = new Map<string, ModelStatus>();
      for (const model of whisperModels) {
        const status = await commands.getModelStatus(model.id);
        statuses.set(model.id, status);
      }
      modelStatuses = statuses;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loadingDevices = false;
    }
  }

  async function handleDeviceChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const deviceId = target.value;

    try {
      await commands.setSelectedDevice(deviceId);
      selectedDevice = deviceId;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleModelChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const modelId = target.value;

    try {
      await commands.setSelectedModel(modelId);
      selectedModel = modelId;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function downloadModel(modelId: string) {
    downloadingModel = modelId;
    downloadProgress = 0;
    error = null;

    try {
      await commands.downloadModel(modelId);

      // Refresh status
      const status = await commands.getModelStatus(modelId);
      modelStatuses.set(modelId, status);
      modelStatuses = modelStatuses; // Trigger reactivity
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      downloadingModel = null;
      downloadProgress = 0;
    }
  }

  async function deleteModel(modelId: string) {
    try {
      await commands.deleteModel(modelId);

      // Refresh status
      const status = await commands.getModelStatus(modelId);
      modelStatuses.set(modelId, status);
      modelStatuses = modelStatuses; // Trigger reactivity
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onclose();
    }
  }

  async function handleTimeoutChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const minutes = parseInt(target.value);
    autoLockTimeout = minutes;
    try {
      await invoke('set_lock_timeout', { seconds: minutes * 60 });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleLockNow() {
    await vaultStore.lock();
    onclose();
  }

  async function handleChangePassword() {
    passwordError = null;

    if (newPassword.length < 8) {
      passwordError = 'New password must be at least 8 characters';
      return;
    }

    if (newPassword !== confirmPassword) {
      passwordError = 'Passwords do not match';
      return;
    }

    changingPassword = true;
    try {
      await invoke('change_password', {
        currentPassword,
        newPassword
      });
      // Reset form
      showChangePassword = false;
      currentPassword = '';
      newPassword = '';
      confirmPassword = '';
    } catch (e) {
      passwordError = e instanceof Error ? e.message : String(e);
    } finally {
      changingPassword = false;
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onclose();
    }
  }

  function formatSize(mb: number): string {
    if (mb >= 1000) {
      return `${(mb / 1000).toFixed(1)} GB`;
    }
    return `${mb} MB`;
  }

  // Load settings when modal becomes visible
  $effect(() => {
    if (visible) {
      loadSettings();
    }
  });
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div
    class="overlay"
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
    role="dialog"
    aria-modal="true"
    aria-labelledby="settings-title"
    tabindex="0"
  >
    <div class="settings-panel">
      <header class="panel-header">
        <h2 id="settings-title">Settings</h2>
        <button class="close-btn" onclick={onclose} aria-label="Close">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </header>

      <div class="settings-content">
        {#if error}
          <div class="error-banner">
            <span>{error}</span>
            <button onclick={() => (error = null)}>Dismiss</button>
          </div>
        {/if}

        <!-- Audio Device Section -->
        <section class="settings-section">
          <h3>Audio Input</h3>
          <p class="section-desc">Select the microphone to use for voice recording</p>

          {#if loadingDevices}
            <div class="loading">Loading devices...</div>
          {:else if audioDevices.length === 0}
            <div class="empty">No audio input devices found</div>
          {:else}
            <select
              class="device-select"
              value={selectedDevice || ""}
              onchange={handleDeviceChange}
            >
              {#each audioDevices as device}
                <option value={device.id}>
                  {device.name} {device.is_default ? "(Default)" : ""}
                </option>
              {/each}
            </select>
          {/if}

          <button class="refresh-btn" onclick={loadSettings} disabled={loadingDevices}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M23 4v6h-6M1 20v-6h6" />
              <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15" />
            </svg>
            Refresh
          </button>
        </section>

        <!-- Whisper Model Section -->
        <section class="settings-section">
          <h3>Transcription Model</h3>
          <p class="section-desc">Select the Whisper model for speech-to-text</p>

          <select
            class="model-select"
            value={selectedModel}
            onchange={handleModelChange}
          >
            {#each whisperModels as model}
              {@const status = modelStatuses.get(model.id)}
              <option value={model.id}>
                {model.name} ({formatSize(model.size_mb)}) {status?.downloaded ? "✓" : ""}
              </option>
            {/each}
          </select>

          <div class="models-list">
            {#each whisperModels as model}
              {@const status = modelStatuses.get(model.id)}
              <div class="model-item">
                <div class="model-info">
                  <span class="model-name">{model.name}</span>
                  <span class="model-size">{formatSize(model.size_mb)}</span>
                </div>
                <p class="model-desc">{model.description}</p>

                <div class="model-actions">
                  {#if downloadingModel === model.id}
                    <div class="download-progress">
                      <div class="progress-bar" style="width: {downloadProgress}%"></div>
                      <span class="progress-text">{downloadProgress.toFixed(0)}%</span>
                    </div>
                  {:else if status?.downloaded}
                    <span class="downloaded-badge">Downloaded</span>
                    {#if model.id !== selectedModel}
                      <button class="delete-btn" onclick={() => deleteModel(model.id)}>
                        Delete
                      </button>
                    {/if}
                  {:else}
                    <button class="download-btn" onclick={() => downloadModel(model.id)}>
                      Download
                    </button>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </section>

        <!-- Vault Section -->
        <section class="settings-section">
          <h3>Vault Security</h3>
          <p class="section-desc">Configure encryption and auto-lock settings</p>

          <div class="vault-row">
            <label for="autolock">Auto-lock after</label>
            <select
              id="autolock"
              class="timeout-select"
              value={autoLockTimeout}
              onchange={handleTimeoutChange}
            >
              {#each autoLockOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </div>

          <div class="vault-actions">
            <button class="lock-btn" onclick={handleLockNow}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
                <path d="M7 11V7a5 5 0 0110 0v4" />
              </svg>
              Lock Now
            </button>

            <button
              class="change-password-toggle"
              onclick={() => showChangePassword = !showChangePassword}
            >
              {showChangePassword ? 'Cancel' : 'Change Password'}
            </button>
          </div>

          {#if showChangePassword}
            <div class="change-password-form">
              <input
                type="password"
                bind:value={currentPassword}
                placeholder="Current password"
                disabled={changingPassword}
              />
              <input
                type="password"
                bind:value={newPassword}
                placeholder="New password (min 8 chars)"
                disabled={changingPassword}
              />
              <input
                type="password"
                bind:value={confirmPassword}
                placeholder="Confirm new password"
                disabled={changingPassword}
              />
              {#if passwordError}
                <p class="password-error">{passwordError}</p>
              {/if}
              <button
                class="save-password-btn"
                onclick={handleChangePassword}
                disabled={changingPassword || !currentPassword || !newPassword || !confirmPassword}
              >
                {changingPassword ? 'Changing...' : 'Save New Password'}
              </button>
            </div>
          {/if}
        </section>

        <!-- Appearance Section -->
        <section class="settings-section">
          <h3>Appearance</h3>
          <p class="section-desc">Choose your preferred color theme</p>

          <div class="theme-grid">
            {#each themeStore.themes as theme}
              <button
                class="theme-card"
                class:selected={themeStore.currentThemeId === theme.id}
                onclick={() => themeStore.setTheme(theme.id)}
              >
                <div class="theme-preview" style="
                  background: {theme.colors.surface0};
                  border-color: {theme.colors.borderDefault};
                ">
                  <div class="preview-accent" style="background: {theme.colors.accent}"></div>
                  <div class="preview-text" style="color: {theme.colors.textPrimary}">Aa</div>
                </div>
                <span class="theme-name">{theme.name}</span>
                <span class="theme-type">{theme.type}</span>
              </button>
            {/each}
          </div>
        </section>
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
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .settings-panel {
    background: var(--surface-2);
    border: 1px solid var(--border-default);
    border-radius: 12px;
    width: 480px;
    max-width: 90vw;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
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
    flex-shrink: 0;
  }

  .panel-header h2 {
    font-size: var(--font-size-base);
    font-weight: 600;
    color: var(--text-primary);
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

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-lg);
  }

  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) var(--space-md);
    background: rgba(255, 107, 107, 0.1);
    border: 1px solid rgba(255, 107, 107, 0.3);
    border-radius: 6px;
    margin-bottom: var(--space-md);
    color: var(--error);
    font-size: var(--font-size-sm);
  }

  .error-banner button {
    font-size: var(--font-size-xs);
    color: var(--error);
    opacity: 0.7;
  }

  .error-banner button:hover {
    opacity: 1;
  }

  .settings-section {
    margin-bottom: var(--space-xl);
  }

  .settings-section h3 {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--space-xs);
  }

  .section-desc {
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    margin-bottom: var(--space-md);
  }

  .loading,
  .empty {
    padding: var(--space-md);
    text-align: center;
    color: var(--text-disabled);
    font-size: var(--font-size-sm);
  }

  .device-select,
  .model-select {
    width: 100%;
    padding: var(--space-sm) var(--space-md);
    background: var(--surface-1);
    border: 1px solid var(--border-default);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23a06048' stroke-width='2'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 12px center;
    padding-right: 36px;
  }

  .device-select option,
  .model-select option {
    background: var(--surface-1);
    color: var(--text-primary);
    padding: var(--space-sm);
  }

  .device-select:focus,
  .model-select:focus {
    outline: none;
    border-color: var(--accent);
  }

  /* Light theme dropdown arrow fix */
  :global([data-theme-type="light"]) .device-select,
  :global([data-theme-type="light"]) .model-select {
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%235f6368' stroke-width='2'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E");
  }

  .refresh-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--space-xs);
    margin-top: var(--space-sm);
    padding: var(--space-xs) var(--space-sm);
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    border-radius: 4px;
    transition: all var(--transition-fast);
  }

  .refresh-btn:hover:not(:disabled) {
    background: var(--surface-3);
    color: var(--text-primary);
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .models-list {
    margin-top: var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .model-item {
    padding: var(--space-md);
    background: var(--surface-1);
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
  }

  .model-info {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-xs);
  }

  .model-name {
    font-size: var(--font-size-sm);
    font-weight: 500;
    color: var(--text-primary);
  }

  .model-size {
    font-size: var(--font-size-xs);
    color: var(--text-disabled);
  }

  .model-desc {
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    margin-bottom: var(--space-sm);
  }

  .model-actions {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .download-btn,
  .delete-btn {
    padding: var(--space-xs) var(--space-sm);
    font-size: var(--font-size-xs);
    border-radius: 4px;
    transition: all var(--transition-fast);
  }

  .download-btn {
    background: var(--accent);
    color: white;
  }

  .download-btn:hover {
    background: var(--accent-hover);
  }

  .delete-btn {
    color: var(--text-secondary);
  }

  .delete-btn:hover {
    color: var(--error);
    background: rgba(255, 107, 107, 0.1);
  }

  .downloaded-badge {
    font-size: var(--font-size-xs);
    color: var(--success);
    padding: 2px 8px;
    background: rgba(81, 207, 102, 0.1);
    border-radius: 4px;
  }

  .download-progress {
    flex: 1;
    height: 20px;
    background: var(--surface-2);
    border-radius: 4px;
    position: relative;
    overflow: hidden;
  }

  .progress-bar {
    height: 100%;
    background: var(--accent);
    transition: width 100ms ease;
  }

  .progress-text {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--font-size-xs);
    color: var(--text-primary);
  }

  .panel-footer {
    padding: var(--space-sm) var(--space-lg);
    border-top: 1px solid var(--border-subtle);
    text-align: center;
    flex-shrink: 0;
  }

  .hint {
    font-size: var(--font-size-xs);
    color: var(--text-disabled);
  }

  .hint kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    color: var(--text-secondary);
    background: var(--surface-0);
    border: 1px solid var(--border-subtle);
    border-radius: 3px;
  }

  /* Theme picker styles */
  .theme-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-sm);
  }

  .theme-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-sm);
    background: var(--surface-1);
    border: 2px solid var(--border-subtle);
    border-radius: 8px;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .theme-card:hover {
    border-color: var(--border-default);
  }

  .theme-card.selected {
    border-color: var(--accent);
    box-shadow: 0 0 12px var(--accent-glow);
  }

  .theme-preview {
    width: 100%;
    height: 48px;
    border-radius: 4px;
    border: 1px solid;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }

  .preview-accent {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 4px;
  }

  .preview-text {
    font-family: var(--font-display);
    font-size: 18px;
  }

  .theme-name {
    font-size: var(--font-size-xs);
    color: var(--text-primary);
    font-weight: 500;
  }

  .theme-type {
    font-size: 10px;
    color: var(--text-disabled);
    text-transform: uppercase;
  }

  /* Vault settings styles */
  .vault-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-md);
  }

  .vault-row label {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  .timeout-select {
    padding: var(--space-xs) var(--space-sm);
    background: var(--surface-1);
    border: 1px solid var(--border-default);
    border-radius: 4px;
    color: var(--text-primary);
    font-size: var(--font-size-sm);
  }

  .vault-actions {
    display: flex;
    gap: var(--space-sm);
    margin-bottom: var(--space-md);
  }

  .lock-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-sm) var(--space-md);
    background: var(--accent);
    color: var(--surface-0);
    font-size: var(--font-size-sm);
    border-radius: 4px;
    transition: all var(--transition-fast);
  }

  .lock-btn:hover {
    background: var(--accent-hover);
  }

  .change-password-toggle {
    padding: var(--space-sm) var(--space-md);
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    border-radius: 4px;
    transition: all var(--transition-fast);
  }

  .change-password-toggle:hover {
    background: var(--surface-3);
    color: var(--text-primary);
  }

  .change-password-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    padding: var(--space-md);
    background: var(--surface-1);
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
  }

  .change-password-form input {
    width: 100%;
    padding: var(--space-sm) var(--space-md);
    background: var(--surface-2);
    border: 1px solid var(--border-default);
    border-radius: 4px;
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    box-sizing: border-box;
  }

  .change-password-form input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .password-error {
    color: var(--error);
    font-size: var(--font-size-xs);
    margin: 0;
  }

  .save-password-btn {
    padding: var(--space-sm) var(--space-md);
    background: var(--accent);
    color: var(--surface-0);
    font-size: var(--font-size-sm);
    border-radius: 4px;
    transition: all var(--transition-fast);
  }

  .save-password-btn:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .save-password-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
