<script lang="ts">
  import { vaultStore } from '../stores/vault.svelte';

  let step = $state(1);
  let password = $state('');
  let confirmPassword = $state('');
  let loading = $state(false);
  let passwordError = $state<string | null>(null);
  let copiedRecoveryKey = $state(false);
  let confirmedSaved = $state(false);

  $effect(() => {
    // Validate passwords match
    if (confirmPassword && password !== confirmPassword) {
      passwordError = 'Passwords do not match';
    } else if (password && password.length < 8) {
      passwordError = 'Password must be at least 8 characters';
    } else {
      passwordError = null;
    }
  });

  async function handleSetup() {
    if (passwordError || !password || password !== confirmPassword) return;
    loading = true;
    try {
      await vaultStore.setup(password);
      password = '';
      confirmPassword = '';
      step = 2;
    } finally {
      loading = false;
    }
  }

  async function copyRecoveryKey() {
    if (vaultStore.recoveryKey) {
      await navigator.clipboard.writeText(vaultStore.recoveryKey);
      copiedRecoveryKey = true;
    }
  }

  function finish() {
    vaultStore.clearRecoveryKey();
  }
</script>

<div class="setup-wizard">
  <div class="wizard-container">
    <h1>ghostnote</h1>
    <p class="subtitle">Encrypted Voice Notes</p>

    <div class="steps">
      <span class:active={step === 1}>1</span>
      <span class="line" class:complete={step > 1}></span>
      <span class:active={step === 2}>2</span>
      <span class="line" class:complete={step > 2}></span>
      <span class:active={step === 3}>3</span>
    </div>

    {#if step === 1}
      <div class="step-content">
        <h2>Create Master Password</h2>
        <p class="description">
          This password encrypts all your notes. Choose a strong password you'll remember.
        </p>

        <input
          type="password"
          bind:value={password}
          placeholder="Password (min 8 characters)"
          disabled={loading}
        />

        <input
          type="password"
          bind:value={confirmPassword}
          placeholder="Confirm password"
          disabled={loading}
        />

        {#if passwordError}
          <p class="error">{passwordError}</p>
        {/if}

        <button
          onclick={handleSetup}
          disabled={loading || !!passwordError || !password || password !== confirmPassword}
        >
          {loading ? 'Setting up...' : 'Create Vault'}
        </button>
      </div>
    {:else if step === 2}
      <div class="step-content">
        <h2>Save Recovery Key</h2>
        <p class="description warning">
          This is your ONLY way to recover your notes if you forget your password.
          Save it somewhere safe.
        </p>

        <div class="recovery-key">
          <code>{vaultStore.recoveryKey}</code>
          <button class="copy" onclick={copyRecoveryKey}>
            {copiedRecoveryKey ? 'Copied!' : 'Copy'}
          </button>
        </div>

        <button onclick={() => step = 3} disabled={!copiedRecoveryKey}>
          I've Saved It
        </button>
      </div>
    {:else}
      <div class="step-content">
        <h2>Confirm</h2>
        <p class="description">
          Please confirm you've saved your recovery key in a safe place.
        </p>

        <label class="checkbox">
          <input type="checkbox" bind:checked={confirmedSaved} />
          I understand that without my password or recovery key, my notes cannot be recovered.
        </label>

        <button onclick={finish} disabled={!confirmedSaved}>
          Start Using ghostnote
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .setup-wizard {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-primary);
    z-index: 9999;
  }

  .wizard-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.5rem;
    padding: 2rem;
    max-width: 450px;
    text-align: center;
  }

  h1 {
    color: var(--text-primary);
    font-size: 2rem;
    margin: 0;
  }

  h2 {
    color: var(--text-primary);
    font-size: 1.25rem;
    margin: 0;
  }

  .subtitle {
    color: var(--text-secondary);
    font-size: 0.875rem;
    margin: 0;
  }

  .steps {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin: 1rem 0;
  }

  .steps span {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--bg-secondary);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .steps span.active {
    background: var(--accent);
    color: var(--bg-primary);
  }

  .steps .line {
    width: 40px;
    height: 2px;
    background: var(--bg-secondary);
    border-radius: 0;
  }

  .steps .line.complete {
    background: var(--accent);
  }

  .step-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    width: 100%;
  }

  .description {
    color: var(--text-secondary);
    font-size: 0.875rem;
    margin: 0;
    line-height: 1.5;
  }

  .description.warning {
    color: #ffaa44;
  }

  input[type="password"] {
    width: 100%;
    padding: 0.75rem 1rem;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 1rem;
    box-sizing: border-box;
  }

  button {
    width: 100%;
    padding: 0.75rem 1rem;
    border: none;
    border-radius: 4px;
    background: var(--accent);
    color: var(--bg-primary);
    font-size: 1rem;
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error {
    color: #ff4444;
    font-size: 0.875rem;
    margin: 0;
  }

  .recovery-key {
    width: 100%;
    padding: 1rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .recovery-key code {
    font-family: monospace;
    font-size: 1rem;
    color: var(--accent);
    word-break: break-all;
    line-height: 1.5;
  }

  .recovery-key .copy {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--text-secondary);
    font-size: 0.875rem;
    padding: 0.5rem;
    width: auto;
  }

  .checkbox {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    text-align: left;
    color: var(--text-secondary);
    font-size: 0.875rem;
    cursor: pointer;
  }

  .checkbox input {
    margin-top: 2px;
    cursor: pointer;
  }
</style>
