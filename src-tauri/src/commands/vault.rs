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
