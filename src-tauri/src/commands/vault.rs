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
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use zeroize::{Zeroize, ZeroizeOnDrop};

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

/// Argon2id parameters (OWASP recommendations for password hashing)
const ARGON2_M_COST: u32 = 65536; // 64 MB memory
const ARGON2_T_COST: u32 = 3; // 3 iterations
const ARGON2_P_COST: u32 = 4; // 4 parallel lanes

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

const NONCE_SIZE: usize = 12;

/// Encrypt data with AES-256-GCM
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| format!("Invalid key: {}", e))?;

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

    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| format!("Invalid key: {}", e))?;

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

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

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
        RecoveryKey(input.replace('-', "").replace(' ', ""))
    }
}

/// Recovery data stored encrypted in vault
#[derive(serde::Serialize, serde::Deserialize)]
pub struct RecoveryData {
    pub kek_bytes: Vec<u8>, // The actual KEK, encrypted with recovery key
}

impl RecoveryData {
    /// Create recovery data by encrypting the KEK with a recovery-key-derived key
    pub fn create(kek: &Kek, recovery_key: &RecoveryKey, salt: &[u8; 32]) -> Result<Self, String> {
        // Derive a key from the recovery key
        let recovery_kek = Kek::derive(&recovery_key.0.replace('-', ""), salt)?;

        // Encrypt the original KEK with the recovery-derived key
        let encrypted_kek = encrypt(recovery_kek.as_bytes(), kek.as_bytes())?;

        Ok(RecoveryData {
            kek_bytes: encrypted_kek,
        })
    }

    /// Recover the KEK using the recovery key
    pub fn recover_kek(&self, recovery_key: &RecoveryKey, salt: &[u8; 32]) -> Result<Kek, String> {
        let recovery_kek = Kek::derive(&recovery_key.0.replace('-', ""), salt)?;
        let kek_bytes = decrypt(recovery_kek.as_bytes(), &self.kek_bytes)?;

        if kek_bytes.len() != 32 {
            return Err("Invalid KEK size".to_string());
        }

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&kek_bytes);
        Ok(Kek(arr))
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
