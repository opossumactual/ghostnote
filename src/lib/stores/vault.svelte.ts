import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface VaultStatus {
  initialized: boolean;
  locked: boolean;
  timeout_remaining: number;
}

let status = $state<VaultStatus>({
  initialized: false,
  locked: true,
  timeout_remaining: 0,
});

let error = $state<string | null>(null);
let recoveryKey = $state<string | null>(null);

export const vaultStore = {
  get status() { return status; },
  get error() { return error; },
  get recoveryKey() { return recoveryKey; },

  async checkStatus() {
    try {
      status = await invoke<VaultStatus>('get_vault_status');
    } catch (e) {
      error = String(e);
    }
  },

  async setup(password: string) {
    try {
      const result = await invoke<{ recovery_key: string }>('setup_vault', { password });
      recoveryKey = result.recovery_key;
      await this.checkStatus();
    } catch (e) {
      error = String(e);
      throw e;
    }
  },

  async unlock(password: string) {
    try {
      await invoke('unlock_vault', { password });
      error = null;
      await this.checkStatus();
    } catch (e) {
      error = String(e);
      throw e;
    }
  },

  async lock() {
    await invoke('lock_vault');
    await this.checkStatus();
  },

  async recover(recoveryKeyInput: string, newPassword: string) {
    try {
      const result = await invoke<{ recovery_key: string }>('recover_vault', {
        recoveryKeyInput,
        newPassword,
      });
      recoveryKey = result.recovery_key;
      await this.checkStatus();
    } catch (e) {
      error = String(e);
      throw e;
    }
  },

  clearRecoveryKey() {
    recoveryKey = null;
  },

  clearError() {
    error = null;
  },
};

// Listen for auto-lock events
listen('vault-locked', () => {
  vaultStore.checkStatus();
});
