//! AK storage with TTL (spec A.7).
//!
//! Strategy: try OS keychain first, fall back to encrypted file.
//! Fallback to env var `DOTMAGE_AK` for headless/CI.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const SERVICE: &str = "dotmage";

#[derive(Serialize, Deserialize)]
struct AkStore {
    ak: String, // base64
    expiry: u64,
}

fn ak_file_path(server_hash: &str) -> PathBuf {
    crate::config::Config::default_dir().join(format!("ak_{server_hash}.json"))
}

/// Store AK with expiry. Tries OS keychain, falls back to file.
pub fn store_ak(server_hash: &str, ak: &[u8; 32], ttl_secs: u64) -> Result<(), KeychainError> {
    let ak_b64 = B64.encode(ak);
    let expiry = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + ttl_secs;

    // Try OS keychain
    if try_store_keyring(server_hash, &ak_b64, expiry) {
        return Ok(());
    }

    // Fallback: file
    let store = AkStore { ak: ak_b64, expiry };
    let path = ak_file_path(server_hash);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string(&store).map_err(|e| KeychainError::Other(e.to_string()))?;
    std::fs::write(&path, data)?;
    Ok(())
}

/// Load AK, checking expiry. Returns None if expired or missing.
pub fn load_ak(server_hash: &str) -> Result<Option<[u8; 32]>, KeychainError> {
    // 1. Env var (CI/headless)
    if let Ok(ak_b64) = std::env::var("DOTMAGE_AK") {
        return decode_ak(&ak_b64).map(Some);
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 2. Try OS keychain
    if let Some(result) = try_load_keyring(server_hash, now) {
        return result.map(Some);
    }

    // 3. Fallback: file
    let path = ak_file_path(server_hash);
    if !path.exists() {
        return Ok(None);
    }
    let data = std::fs::read_to_string(&path)?;
    let store: AkStore =
        serde_json::from_str(&data).map_err(|e| KeychainError::Other(e.to_string()))?;

    if now > store.expiry {
        let _ = std::fs::remove_file(&path);
        return Ok(None);
    }

    decode_ak(&store.ak).map(Some)
}

/// Delete AK from both keychain and file.
pub fn delete_ak(server_hash: &str) -> Result<(), KeychainError> {
    // Keychain
    let ak_account = format!("ak:{server_hash}");
    let expiry_account = format!("ak-expiry:{server_hash}");
    if let Ok(entry) = keyring::Entry::new(SERVICE, &ak_account) {
        let _ = entry.delete_credential();
    }
    if let Ok(entry) = keyring::Entry::new(SERVICE, &expiry_account) {
        let _ = entry.delete_credential();
    }

    // File
    let _ = std::fs::remove_file(ak_file_path(server_hash));

    Ok(())
}

// --- OS Keychain helpers ---

fn try_store_keyring(server_hash: &str, ak_b64: &str, expiry: u64) -> bool {
    let ak_account = format!("ak:{server_hash}");
    let expiry_account = format!("ak-expiry:{server_hash}");

    let ak_entry = match keyring::Entry::new(SERVICE, &ak_account) {
        Ok(e) => e,
        Err(_) => return false,
    };
    if ak_entry.set_password(ak_b64).is_err() {
        return false;
    }

    // Verify round-trip — some macOS configs silently drop the write
    match ak_entry.get_password() {
        Ok(readback) if readback == ak_b64 => {}
        _ => {
            let _ = ak_entry.delete_credential();
            return false;
        }
    }

    let expiry_entry = match keyring::Entry::new(SERVICE, &expiry_account) {
        Ok(e) => e,
        Err(_) => return false,
    };
    if expiry_entry.set_password(&expiry.to_string()).is_err() {
        return false;
    }

    true
}

fn try_load_keyring(server_hash: &str, now: u64) -> Option<Result<[u8; 32], KeychainError>> {
    let ak_account = format!("ak:{server_hash}");
    let expiry_account = format!("ak-expiry:{server_hash}");

    let expiry_entry = keyring::Entry::new(SERVICE, &expiry_account).ok()?;
    let expiry_str = expiry_entry.get_password().ok()?;
    let expiry: u64 = expiry_str.parse().ok()?;

    if now > expiry {
        let _ = delete_ak(server_hash);
        return None;
    }

    let ak_entry = keyring::Entry::new(SERVICE, &ak_account).ok()?;
    let ak_b64 = ak_entry.get_password().ok()?;
    Some(decode_ak(&ak_b64))
}

// --- Shared ---

fn decode_ak(b64: &str) -> Result<[u8; 32], KeychainError> {
    let bytes = B64
        .decode(b64)
        .map_err(|e| KeychainError::Other(e.to_string()))?;
    bytes
        .try_into()
        .map_err(|_| KeychainError::Other("AK must be 32 bytes".into()))
}

/// Compute a short hash of server URL for keychain account naming.
pub fn server_hash(url: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(url.as_bytes());
    hex::encode(&hash[..8])
}

#[derive(Debug, thiserror::Error)]
pub enum KeychainError {
    #[error("keychain: not found")]
    NotFound,
    #[error("keychain: {0}")]
    Other(String),
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
}
