# ghostnote Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform onote into ghostnote, a hardened voice notes app with AES-256-GCM encryption, master password protection, and forensic resistance.

**Architecture:** All notes encrypted at rest with per-note keys (DEKs) wrapped by a master key (KEK) derived from password via Argon2id. Auto-lock after inactivity clears keys from memory. Audio transcribed in memory then discarded (never written to disk).

**Tech Stack:** Tauri v2, Svelte 5 (runes), Rust (aes-gcm, argon2, zeroize crates), whisper-rs

**Design Doc:** `docs/plans/2025-01-02-ghostnote-design.md`

---

## Phase 1: Rust Encryption Foundation

### Task 1: Add Encryption Dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`

**Step 1: Add crates to Cargo.toml**

Add these dependencies under `[dependencies]`:

```toml
aes-gcm = "0.10"
argon2 = "0.5"
zeroize = { version = "1.8", features = ["derive"] }
rand = "0.8"
base64 = "0.22"
```

**Step 2: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles with new dependencies

**Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "chore: add encryption dependencies (aes-gcm, argon2, zeroize)"
```

---

### Task 2: Create Vault Module Structure

**Files:**
- Create: `src-tauri/src/commands/vault.rs`
- Modify: `src-tauri/src/commands/mod.rs`

**Step 1: Create vault.rs with module structure**

```rust
//! Vault management for ghostnote encryption
//!
//! Handles:
//! - Master password / KEK derivation
//! - Per-note DEK generation and wrapping
//! - Encryption/decryption of note content

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use zeroize::{Zeroize, ZeroizeOnDrop};
use rand::RngCore;
use std::path::PathBuf;

/// Key Encryption Key - derived from password, wraps DEKs
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Kek([u8; 32]);

/// Data Encryption Key - unique per note, encrypts content
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Dek([u8; 32]);

/// Vault configuration and paths
pub struct VaultConfig {
    pub vault_dir: PathBuf,
    pub salt_path: PathBuf,
    pub verify_path: PathBuf,
    pub recovery_path: PathBuf,
}

impl VaultConfig {
    pub fn new(base_dir: &PathBuf) -> Self {
        let vault_dir = base_dir.join(".vault");
        Self {
            salt_path: vault_dir.join("salt"),
            verify_path: vault_dir.join("verify"),
            recovery_path: vault_dir.join("recovery.key"),
            vault_dir,
        }
    }
}

/// Check if vault is initialized (has salt file)
pub fn is_vault_initialized(config: &VaultConfig) -> bool {
    config.salt_path.exists()
}
```

**Step 2: Export vault module in mod.rs**

Add to `src-tauri/src/commands/mod.rs`:

```rust
pub mod vault;
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src-tauri/src/commands/vault.rs src-tauri/src/commands/mod.rs
git commit -m "feat(vault): add vault module structure with Kek/Dek types"
```

---

### Task 3: Implement Key Derivation

**Files:**
- Modify: `src-tauri/src/commands/vault.rs`

**Step 1: Add key derivation functions**

Add after the existing code in vault.rs:

```rust
use argon2::{Algorithm, Params, Version};

/// Argon2id parameters (OWASP recommendations for password hashing)
const ARGON2_M_COST: u32 = 65536;  // 64 MB memory
const ARGON2_T_COST: u32 = 3;      // 3 iterations
const ARGON2_P_COST: u32 = 4;      // 4 parallel lanes

impl Kek {
    /// Derive KEK from password and salt using Argon2id
    pub fn derive(password: &str, salt: &[u8; 32]) -> Result<Self, String> {
        let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(32))
            .map_err(|e| format!("Invalid Argon2 params: {}", e))?;

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut kek_bytes = [0u8; 32];
        argon2
            .hash_password_into(password.as_bytes(), salt, &mut kek_bytes)
            .map_err(|e| format!("Key derivation failed: {}", e))?;

        Ok(Kek(kek_bytes))
    }

    /// Get the raw key bytes (for AES operations)
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Dek {
    /// Generate a new random DEK
    pub fn generate() -> Self {
        let mut dek_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut dek_bytes);
        Dek(dek_bytes)
    }

    /// Get the raw key bytes (for AES operations)
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Create DEK from raw bytes (after unwrapping)
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Dek(bytes)
    }
}

/// Generate a random 32-byte salt
pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    salt
}
```

**Step 2: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src-tauri/src/commands/vault.rs
git commit -m "feat(vault): implement Argon2id key derivation"
```

---

### Task 4: Implement Encryption/Decryption

**Files:**
- Modify: `src-tauri/src/commands/vault.rs`

**Step 1: Add encryption functions**

Add after key derivation code:

```rust
const NONCE_SIZE: usize = 12;

/// Encrypt data with AES-256-GCM
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Invalid key: {}", e))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt data with AES-256-GCM
pub fn decrypt(key: &[u8; 32], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    if ciphertext.len() < NONCE_SIZE {
        return Err("Ciphertext too short".to_string());
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Invalid key: {}", e))?;

    let nonce = Nonce::from_slice(&ciphertext[..NONCE_SIZE]);
    let encrypted_data = &ciphertext[NONCE_SIZE..];

    cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))
}

/// Wrap a DEK with the KEK (encrypt the DEK)
pub fn wrap_dek(kek: &Kek, dek: &Dek) -> Result<Vec<u8>, String> {
    encrypt(kek.as_bytes(), dek.as_bytes())
}

/// Unwrap a DEK with the KEK (decrypt the DEK)
pub fn unwrap_dek(kek: &Kek, wrapped_dek: &[u8]) -> Result<Dek, String> {
    let dek_bytes = decrypt(kek.as_bytes(), wrapped_dek)?;
    if dek_bytes.len() != 32 {
        return Err("Invalid DEK size".to_string());
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&dek_bytes);
    Ok(Dek::from_bytes(arr))
}
```

**Step 2: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src-tauri/src/commands/vault.rs
git commit -m "feat(vault): implement AES-256-GCM encryption/decryption"
```

---

### Task 5: Implement Recovery Key Generation

**Files:**
- Modify: `src-tauri/src/commands/vault.rs`

**Step 1: Add recovery key functions**

Add after encryption code:

```rust
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Recovery key format: XXXX-XXXX-XXXX-XXXX-XXXX-XXXX (24 chars of base64)
pub struct RecoveryKey(String);

impl RecoveryKey {
    /// Generate a new random recovery key
    pub fn generate() -> Self {
        let mut bytes = [0u8; 18]; // 18 bytes = 24 base64 chars
        OsRng.fill_bytes(&mut bytes);
        let encoded = BASE64.encode(bytes);

        // Format as XXXX-XXXX-XXXX-XXXX-XXXX-XXXX
        let formatted = encoded
            .chars()
            .collect::<Vec<_>>()
            .chunks(4)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("-");

        RecoveryKey(formatted)
    }

    /// Get the display string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Parse from user input (removes dashes)
    pub fn from_input(input: &str) -> Self {
        RecoveryKey(input.replace("-", "").replace(" ", ""))
    }

    /// Get raw bytes for encryption operations
    pub fn as_bytes(&self) -> Vec<u8> {
        self.0.replace("-", "").into_bytes()
    }
}

/// Recovery data stored encrypted in vault
#[derive(serde::Serialize, serde::Deserialize)]
pub struct RecoveryData {
    pub kek_bytes: Vec<u8>,  // The actual KEK, encrypted with recovery key
}

impl RecoveryData {
    /// Create recovery data by encrypting the KEK with a recovery-key-derived key
    pub fn create(kek: &Kek, recovery_key: &RecoveryKey, salt: &[u8; 32]) -> Result<Self, String> {
        // Derive a key from the recovery key
        let recovery_kek = Kek::derive(&recovery_key.0.replace("-", ""), salt)?;

        // Encrypt the original KEK with the recovery-derived key
        let encrypted_kek = encrypt(recovery_kek.as_bytes(), kek.as_bytes())?;

        Ok(RecoveryData {
            kek_bytes: encrypted_kek,
        })
    }

    /// Recover the KEK using the recovery key
    pub fn recover_kek(&self, recovery_key: &RecoveryKey, salt: &[u8; 32]) -> Result<Kek, String> {
        let recovery_kek = Kek::derive(&recovery_key.0.replace("-", ""), salt)?;
        let kek_bytes = decrypt(recovery_kek.as_bytes(), &self.kek_bytes)?;

        if kek_bytes.len() != 32 {
            return Err("Invalid KEK size".to_string());
        }

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&kek_bytes);
        Ok(Kek(arr))
    }
}
```

**Step 2: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src-tauri/src/commands/vault.rs
git commit -m "feat(vault): implement recovery key generation and KEK recovery"
```

---

## Phase 2: Vault State Management

### Task 6: Create Vault State

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands/vault.rs`

**Step 1: Add VaultState to vault.rs**

Add at the end of vault.rs:

```rust
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Thread-safe vault state
pub struct VaultState {
    inner: Mutex<VaultStateInner>,
}

struct VaultStateInner {
    kek: Option<Kek>,
    config: Option<VaultConfig>,
    last_activity: Instant,
    lock_timeout: Duration,
}

impl Default for VaultState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(VaultStateInner {
                kek: None,
                config: None,
                last_activity: Instant::now(),
                lock_timeout: Duration::from_secs(300), // 5 minutes default
            }),
        }
    }
}

impl VaultState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize vault config (called on app start)
    pub fn set_config(&self, config: VaultConfig) {
        let mut inner = self.inner.lock().unwrap();
        inner.config = Some(config);
    }

    /// Check if vault is unlocked
    pub fn is_unlocked(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.kek.is_some()
    }

    /// Unlock vault with KEK
    pub fn unlock(&self, kek: Kek) {
        let mut inner = self.inner.lock().unwrap();
        inner.kek = Some(kek);
        inner.last_activity = Instant::now();
    }

    /// Lock vault (clear KEK from memory)
    pub fn lock(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.kek = None; // Zeroize will clear memory
    }

    /// Record user activity (resets auto-lock timer)
    pub fn touch(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.last_activity = Instant::now();
    }

    /// Set auto-lock timeout
    pub fn set_timeout(&self, seconds: u64) {
        let mut inner = self.inner.lock().unwrap();
        inner.lock_timeout = Duration::from_secs(seconds);
    }

    /// Get time remaining until auto-lock (in seconds)
    pub fn time_until_lock(&self) -> u64 {
        let inner = self.inner.lock().unwrap();
        let elapsed = inner.last_activity.elapsed();
        if elapsed >= inner.lock_timeout {
            0
        } else {
            (inner.lock_timeout - elapsed).as_secs()
        }
    }

    /// Check if should auto-lock
    pub fn should_lock(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.kek.is_some() && inner.last_activity.elapsed() >= inner.lock_timeout
    }

    /// Execute operation with KEK (returns error if locked)
    pub fn with_kek<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&Kek) -> Result<T, String>,
    {
        let inner = self.inner.lock().unwrap();
        match &inner.kek {
            Some(kek) => f(kek),
            None => Err("Vault is locked".to_string()),
        }
    }

    /// Get vault config
    pub fn config(&self) -> Result<VaultConfig, String> {
        let inner = self.inner.lock().unwrap();
        inner.config.clone().ok_or_else(|| "Vault not configured".to_string())
    }
}

// Make VaultConfig cloneable
impl Clone for VaultConfig {
    fn clone(&self) -> Self {
        Self {
            vault_dir: self.vault_dir.clone(),
            salt_path: self.salt_path.clone(),
            verify_path: self.verify_path.clone(),
            recovery_path: self.recovery_path.clone(),
        }
    }
}
```

**Step 2: Add VaultState to Tauri app state in lib.rs**

First, read the current lib.rs to understand its structure, then add VaultState.

Find the `run()` function and add VaultState to the managed state. Add this import at the top:

```rust
use commands::vault::VaultState;
```

And in the builder chain, add:

```rust
.manage(VaultState::new())
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src-tauri/src/commands/vault.rs src-tauri/src/lib.rs
git commit -m "feat(vault): add thread-safe VaultState with auto-lock timer"
```

---

### Task 7: Implement Vault Setup Command

**Files:**
- Modify: `src-tauri/src/commands/vault.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add setup_vault command**

Add to vault.rs:

```rust
use std::fs;

#[derive(serde::Serialize)]
pub struct SetupResult {
    pub recovery_key: String,
}

/// Initialize a new vault with password
#[tauri::command]
pub async fn setup_vault(
    password: String,
    state: tauri::State<'_, VaultState>,
) -> Result<SetupResult, String> {
    let config = state.config()?;

    // Create vault directory
    fs::create_dir_all(&config.vault_dir)
        .map_err(|e| format!("Failed to create vault directory: {}", e))?;

    // Generate salt
    let salt = generate_salt();
    fs::write(&config.salt_path, &salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;

    // Derive KEK from password
    let kek = Kek::derive(&password, &salt)?;

    // Create verification blob (encrypt a known string)
    let verify_plaintext = b"ghostnote-verify";
    let verify_encrypted = encrypt(kek.as_bytes(), verify_plaintext)?;
    fs::write(&config.verify_path, &verify_encrypted)
        .map_err(|e| format!("Failed to write verify blob: {}", e))?;

    // Generate and store recovery key
    let recovery_key = RecoveryKey::generate();
    let recovery_data = RecoveryData::create(&kek, &recovery_key, &salt)?;
    let recovery_json = serde_json::to_vec(&recovery_data)
        .map_err(|e| format!("Failed to serialize recovery data: {}", e))?;
    fs::write(&config.recovery_path, &recovery_json)
        .map_err(|e| format!("Failed to write recovery key: {}", e))?;

    // Unlock vault
    state.unlock(kek);

    Ok(SetupResult {
        recovery_key: recovery_key.as_str().to_string(),
    })
}

/// Check if vault is initialized
#[tauri::command]
pub async fn is_vault_setup(
    state: tauri::State<'_, VaultState>,
) -> Result<bool, String> {
    let config = state.config()?;
    Ok(is_vault_initialized(&config))
}
```

**Step 2: Register command in lib.rs**

Add `setup_vault` and `is_vault_setup` to the invoke_handler:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::vault::setup_vault,
    commands::vault::is_vault_setup,
])
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src-tauri/src/commands/vault.rs src-tauri/src/lib.rs
git commit -m "feat(vault): add setup_vault and is_vault_setup commands"
```

---

### Task 8: Implement Unlock/Lock Commands

**Files:**
- Modify: `src-tauri/src/commands/vault.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add unlock_vault command**

Add to vault.rs:

```rust
/// Unlock vault with password
#[tauri::command]
pub async fn unlock_vault(
    password: String,
    state: tauri::State<'_, VaultState>,
) -> Result<(), String> {
    let config = state.config()?;

    // Read salt
    let salt_bytes = fs::read(&config.salt_path)
        .map_err(|e| format!("Failed to read salt: {}", e))?;
    if salt_bytes.len() != 32 {
        return Err("Invalid salt file".to_string());
    }
    let mut salt = [0u8; 32];
    salt.copy_from_slice(&salt_bytes);

    // Derive KEK
    let kek = Kek::derive(&password, &salt)?;

    // Verify password by decrypting verify blob
    let verify_encrypted = fs::read(&config.verify_path)
        .map_err(|e| format!("Failed to read verify blob: {}", e))?;
    let verify_decrypted = decrypt(kek.as_bytes(), &verify_encrypted)
        .map_err(|_| "Wrong password".to_string())?;

    if verify_decrypted != b"ghostnote-verify" {
        return Err("Wrong password".to_string());
    }

    // Unlock
    state.unlock(kek);
    Ok(())
}

/// Lock vault
#[tauri::command]
pub async fn lock_vault(
    state: tauri::State<'_, VaultState>,
) -> Result<(), String> {
    state.lock();
    Ok(())
}

/// Get vault lock status
#[tauri::command]
pub async fn get_vault_status(
    state: tauri::State<'_, VaultState>,
) -> Result<VaultStatus, String> {
    let config = state.config()?;
    Ok(VaultStatus {
        initialized: is_vault_initialized(&config),
        locked: !state.is_unlocked(),
        timeout_remaining: state.time_until_lock(),
    })
}

#[derive(serde::Serialize)]
pub struct VaultStatus {
    pub initialized: bool,
    pub locked: bool,
    pub timeout_remaining: u64,
}

/// Record activity (reset auto-lock timer)
#[tauri::command]
pub async fn vault_activity(
    state: tauri::State<'_, VaultState>,
) -> Result<(), String> {
    state.touch();
    Ok(())
}
```

**Step 2: Register commands in lib.rs**

Add to invoke_handler:

```rust
commands::vault::unlock_vault,
commands::vault::lock_vault,
commands::vault::get_vault_status,
commands::vault::vault_activity,
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src-tauri/src/commands/vault.rs src-tauri/src/lib.rs
git commit -m "feat(vault): add unlock, lock, and status commands"
```

---

### Task 9: Implement Password Recovery

**Files:**
- Modify: `src-tauri/src/commands/vault.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add recovery command**

Add to vault.rs:

```rust
/// Recover vault with recovery key and set new password
#[tauri::command]
pub async fn recover_vault(
    recovery_key_input: String,
    new_password: String,
    state: tauri::State<'_, VaultState>,
) -> Result<SetupResult, String> {
    let config = state.config()?;

    // Read salt
    let salt_bytes = fs::read(&config.salt_path)
        .map_err(|e| format!("Failed to read salt: {}", e))?;
    if salt_bytes.len() != 32 {
        return Err("Invalid salt file".to_string());
    }
    let mut salt = [0u8; 32];
    salt.copy_from_slice(&salt_bytes);

    // Parse recovery key
    let recovery_key = RecoveryKey::from_input(&recovery_key_input);

    // Read and decrypt recovery data
    let recovery_json = fs::read(&config.recovery_path)
        .map_err(|e| format!("Failed to read recovery data: {}", e))?;
    let recovery_data: RecoveryData = serde_json::from_slice(&recovery_json)
        .map_err(|e| format!("Invalid recovery data: {}", e))?;

    // Recover the original KEK
    let original_kek = recovery_data.recover_kek(&recovery_key, &salt)
        .map_err(|_| "Invalid recovery key".to_string())?;

    // Generate new salt for new password
    let new_salt = generate_salt();
    fs::write(&config.salt_path, &new_salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;

    // Derive new KEK from new password
    let new_kek = Kek::derive(&new_password, &new_salt)?;

    // Re-encrypt verify blob with new KEK
    let verify_plaintext = b"ghostnote-verify";
    let verify_encrypted = encrypt(new_kek.as_bytes(), verify_plaintext)?;
    fs::write(&config.verify_path, &verify_encrypted)
        .map_err(|e| format!("Failed to write verify blob: {}", e))?;

    // Generate new recovery key
    let new_recovery_key = RecoveryKey::generate();
    let new_recovery_data = RecoveryData::create(&new_kek, &new_recovery_key, &new_salt)?;
    let new_recovery_json = serde_json::to_vec(&new_recovery_data)
        .map_err(|e| format!("Failed to serialize recovery data: {}", e))?;
    fs::write(&config.recovery_path, &new_recovery_json)
        .map_err(|e| format!("Failed to write recovery key: {}", e))?;

    // Re-wrap all existing DEKs with new KEK
    // TODO: This requires iterating all .key files and re-wrapping
    // For now, we'll handle this when we integrate with notes

    // Unlock with new KEK
    state.unlock(new_kek);

    Ok(SetupResult {
        recovery_key: new_recovery_key.as_str().to_string(),
    })
}

/// Change password (requires current password)
#[tauri::command]
pub async fn change_password(
    current_password: String,
    new_password: String,
    state: tauri::State<'_, VaultState>,
) -> Result<SetupResult, String> {
    let config = state.config()?;

    // Verify current password first
    unlock_vault(current_password.clone(), state.clone()).await?;

    // Now proceed with password change (similar to recovery but we have the KEK)
    let new_salt = generate_salt();
    fs::write(&config.salt_path, &new_salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;

    let new_kek = Kek::derive(&new_password, &new_salt)?;

    let verify_plaintext = b"ghostnote-verify";
    let verify_encrypted = encrypt(new_kek.as_bytes(), verify_plaintext)?;
    fs::write(&config.verify_path, &verify_encrypted)
        .map_err(|e| format!("Failed to write verify blob: {}", e))?;

    let new_recovery_key = RecoveryKey::generate();
    let new_recovery_data = RecoveryData::create(&new_kek, &new_recovery_key, &new_salt)?;
    let new_recovery_json = serde_json::to_vec(&new_recovery_data)
        .map_err(|e| format!("Failed to serialize recovery data: {}", e))?;
    fs::write(&config.recovery_path, &new_recovery_json)
        .map_err(|e| format!("Failed to write recovery key: {}", e))?;

    state.unlock(new_kek);

    Ok(SetupResult {
        recovery_key: new_recovery_key.as_str().to_string(),
    })
}
```

**Step 2: Register commands in lib.rs**

Add to invoke_handler:

```rust
commands::vault::recover_vault,
commands::vault::change_password,
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src-tauri/src/commands/vault.rs src-tauri/src/lib.rs
git commit -m "feat(vault): add password recovery and change commands"
```

---

## Phase 3: Encrypted Notes Integration

### Task 10: Modify Notes Commands for Encryption

**Files:**
- Modify: `src-tauri/src/commands/notes.rs`

**Step 1: Read current notes.rs**

Read the file to understand current structure before modifying.

**Step 2: Add encryption to save_note**

Modify `save_note` to:
1. Generate DEK if new note
2. Encrypt content with DEK
3. Wrap DEK with KEK
4. Save .enc and .key files

**Step 3: Add decryption to read_note**

Modify `read_note` to:
1. Read wrapped DEK from .key file
2. Unwrap DEK with KEK
3. Decrypt .enc file
4. Return plaintext

**Step 4: Update file paths**

Change from `.md` files to `.enc` + `.key` pairs.

**Step 5: Verify compilation**

Run: `cd src-tauri && cargo check`

**Step 6: Commit**

```bash
git add src-tauri/src/commands/notes.rs
git commit -m "feat(notes): integrate encryption for note storage"
```

---

## Phase 4: Memory-Only Audio

### Task 11: Remove Audio File Writing

**Files:**
- Modify: `src-tauri/src/commands/audio.rs`

**Step 1: Read current audio.rs**

Understand the current audio recording flow.

**Step 2: Remove disk writes**

Remove the WAV file saving. Keep audio in memory buffer only.

**Step 3: Pass buffer directly to whisper**

Modify to return audio buffer instead of file path.

**Step 4: Verify compilation**

Run: `cd src-tauri && cargo check`

**Step 5: Commit**

```bash
git add src-tauri/src/commands/audio.rs
git commit -m "feat(audio): memory-only recording, no disk writes"
```

---

### Task 12: Update Whisper to Accept Buffer

**Files:**
- Modify: `src-tauri/src/commands/whisper.rs`

**Step 1: Modify transcribe to accept audio buffer**

Change from file path input to Vec<f32> samples.

**Step 2: Clear buffer after transcription**

Ensure audio data is zeroed after use.

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`

**Step 4: Commit**

```bash
git add src-tauri/src/commands/whisper.rs
git commit -m "feat(whisper): accept audio buffer directly, clear after use"
```

---

## Phase 5: Auto-Lock System

### Task 13: Implement Auto-Lock Timer

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add background timer task**

Spawn a background task that checks `VaultState::should_lock()` every second.

**Step 2: Emit lock event**

When auto-lock triggers, emit a Tauri event to frontend.

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`

**Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add auto-lock background timer"
```

---

## Phase 6: Frontend - Lock Screen & Setup

### Task 14: Create Vault Store

**Files:**
- Create: `src/lib/stores/vault.svelte.ts`

**Step 1: Create vault store**

```typescript
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
```

**Step 2: Commit**

```bash
git add src/lib/stores/vault.svelte.ts
git commit -m "feat(ui): add vault store for lock state management"
```

---

### Task 15: Create Lock Screen Component

**Files:**
- Create: `src/lib/components/LockScreen.svelte`

**Step 1: Create component**

```svelte
<script lang="ts">
  import { vaultStore } from '../stores/vault.svelte';

  let password = $state('');
  let loading = $state(false);

  async function handleUnlock() {
    if (!password) return;
    loading = true;
    try {
      await vaultStore.unlock(password);
      password = '';
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleUnlock();
    }
  }
</script>

<div class="lock-screen">
  <div class="lock-container">
    <h1>ghostnote</h1>

    <input
      type="password"
      bind:value={password}
      onkeydown={handleKeydown}
      placeholder="Enter password"
      disabled={loading}
      autofocus
    />

    <button onclick={handleUnlock} disabled={loading || !password}>
      {loading ? 'Unlocking...' : 'Unlock'}
    </button>

    {#if vaultStore.error}
      <p class="error">{vaultStore.error}</p>
    {/if}

    <button class="link" onclick={() => { /* show recovery */ }}>
      Forgot password?
    </button>
  </div>
</div>

<style>
  .lock-screen {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-primary);
    z-index: 9999;
  }

  .lock-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 2rem;
  }

  h1 {
    color: var(--text-primary);
    font-size: 2rem;
    margin-bottom: 1rem;
  }

  input {
    width: 300px;
    padding: 0.75rem 1rem;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 1rem;
  }

  button {
    width: 300px;
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

  button.link {
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.875rem;
    width: auto;
  }

  .error {
    color: #ff4444;
    font-size: 0.875rem;
  }
</style>
```

**Step 2: Commit**

```bash
git add src/lib/components/LockScreen.svelte
git commit -m "feat(ui): add lock screen component"
```

---

### Task 16: Create Setup Wizard Component

**Files:**
- Create: `src/lib/components/SetupWizard.svelte`

**Step 1: Create multi-step setup wizard**

Implement 3-step wizard:
1. Create password (with confirmation)
2. Show recovery key (with copy button)
3. Confirmation checkbox and finish

**Step 2: Commit**

```bash
git add src/lib/components/SetupWizard.svelte
git commit -m "feat(ui): add setup wizard component"
```

---

### Task 17: Integrate Lock Screen in App.svelte

**Files:**
- Modify: `src/App.svelte`

**Step 1: Add vault status check on mount**

**Step 2: Conditionally render LockScreen or SetupWizard**

**Step 3: Commit**

```bash
git add src/App.svelte
git commit -m "feat(ui): integrate lock screen and setup wizard in main app"
```

---

## Phase 7: Rebranding & Theme

### Task 18: Rebrand to ghostnote

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src/lib/components/Toolbar.svelte`
- Modify: `package.json`

**Step 1: Update app name in tauri.conf.json**

Change "onote" to "ghostnote" in:
- productName
- identifier
- title

**Step 2: Update Toolbar title**

**Step 3: Update package.json name**

**Step 4: Commit**

```bash
git add src-tauri/tauri.conf.json src/lib/components/Toolbar.svelte package.json
git commit -m "chore: rebrand to ghostnote"
```

---

### Task 19: Create Covert Theme Palette

**Files:**
- Modify: `src/lib/themes.ts`

**Step 1: Add muted theme**

Create a new default theme with:
- Dark grays for backgrounds
- Muted/desaturated accent (slate blue or muted purple)
- Low contrast, covert aesthetic

**Step 2: Set as default**

**Step 3: Commit**

```bash
git add src/lib/themes.ts
git commit -m "feat(theme): add covert muted color palette as default"
```

---

## Phase 8: Settings Integration

### Task 20: Add Vault Settings

**Files:**
- Modify: `src/lib/components/Settings.svelte`

**Step 1: Add auto-lock timeout dropdown**

Options: 1, 5, 10, 15, 30 minutes

**Step 2: Add "Lock Now" button**

**Step 3: Add "Change Password" section**

**Step 4: Commit**

```bash
git add src/lib/components/Settings.svelte
git commit -m "feat(settings): add vault settings (timeout, lock, password change)"
```

---

## Phase 9: Data Directory Migration

### Task 21: Update Data Paths

**Files:**
- Modify: `src-tauri/src/commands/notes.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Change directory from opnotes to ghostnote**

Update: `~/Documents/opnotes/` → `~/Documents/ghostnote/`

**Step 2: Initialize VaultConfig on app start**

**Step 3: Commit**

```bash
git add src-tauri/src/commands/notes.rs src-tauri/src/lib.rs
git commit -m "feat: use ~/Documents/ghostnote/ for data storage"
```

---

## Phase 10: Final Integration & Testing

### Task 22: Wire Up Activity Tracking

**Files:**
- Modify: `src/App.svelte`

**Step 1: Add activity listeners**

Call `vault_activity` on:
- Keystrokes
- Mouse clicks
- Recording start/stop

**Step 2: Commit**

```bash
git add src/App.svelte
git commit -m "feat: track user activity for auto-lock timer"
```

---

### Task 23: End-to-End Testing

**Step 1: Build and run**

```bash
npm run tauri dev
```

**Step 2: Test setup flow**

1. First launch shows setup wizard
2. Create password
3. Recovery key displayed
4. Can copy recovery key

**Step 3: Test lock/unlock**

1. Create a note
2. Wait for auto-lock (or lock manually)
3. Verify note content not accessible
4. Unlock with password
5. Verify note content restored

**Step 4: Test recovery**

1. Lock vault
2. Click "Forgot password"
3. Enter recovery key
4. Set new password
5. Verify can unlock with new password

**Step 5: Verify encryption**

1. Create notes
2. Check `~/Documents/ghostnote/` for `.enc` files
3. Verify files are encrypted (not readable text)

---

### Task 24: Update CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Update documentation**

Reflect ghostnote changes:
- New vault commands
- Encryption architecture
- Changed data paths
- New components

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md for ghostnote architecture"
```

---

## Summary

**Total Tasks:** 24

**Phase breakdown:**
- Phase 1 (Tasks 1-5): Rust encryption foundation
- Phase 2 (Tasks 6-9): Vault state management
- Phase 3 (Task 10): Encrypted notes integration
- Phase 4 (Tasks 11-12): Memory-only audio
- Phase 5 (Task 13): Auto-lock system
- Phase 6 (Tasks 14-17): Frontend lock screen & setup
- Phase 7 (Tasks 18-19): Rebranding & theme
- Phase 8 (Task 20): Settings integration
- Phase 9 (Task 21): Data directory migration
- Phase 10 (Tasks 22-24): Final integration & testing

**Key files created:**
- `src-tauri/src/commands/vault.rs`
- `src/lib/stores/vault.svelte.ts`
- `src/lib/components/LockScreen.svelte`
- `src/lib/components/SetupWizard.svelte`

**Key files modified:**
- `src-tauri/Cargo.toml`
- `src-tauri/src/lib.rs`
- `src-tauri/src/commands/notes.rs`
- `src-tauri/src/commands/audio.rs`
- `src-tauri/src/commands/whisper.rs`
- `src/App.svelte`
- `src/lib/components/Settings.svelte`
- `src/lib/themes.ts`
