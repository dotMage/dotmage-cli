//! OS keychain integration for AK storage (spec A.7).
//!
//! Stores AK in OS-native secure storage with TTL via a separate expiry entry.
//! Fallback to env var `DOTMAGE_AK` for headless/CI.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use std::time::{SystemTime, UNIX_EPOCH};

const SERVICE: &str = "dotmage";

/// Store AK in the OS keychain with expiry.
pub fn store_ak(server_hash: &str, ak: &[u8; 32], ttl_secs: u64) -> Result<(), KeychainError> {
    let ak_account = format!("ak:{server_hash}");
    let expiry_account = format!("ak-expiry:{server_hash}");

    let entry = keyring::Entry::new(SERVICE, &ak_account)
        .map_err(|e| KeychainError::Other(e.to_string()))?;
    entry
        .set_password(&B64.encode(ak))
        .map_err(|e| KeychainError::Other(e.to_string()))?;

    let expiry = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + ttl_secs;

    let expiry_entry = keyring::Entry::new(SERVICE, &expiry_account)
        .map_err(|e| KeychainError::Other(e.to_string()))?;
    expiry_entry
        .set_password(&expiry.to_string())
        .map_err(|e| KeychainError::Other(e.to_string()))?;

    Ok(())
}

/// Load AK from keychain, checking expiry. Returns None if expired or missing.
pub fn load_ak(server_hash: &str) -> Result<Option<[u8; 32]>, KeychainError> {
    // Check env var fallback first (for CI/headless)
    if let Ok(ak_b64) = std::env::var("DOTMAGE_AK") {
        return decode_ak(&ak_b64).map(Some);
    }

    let ak_account = format!("ak:{server_hash}");
    let expiry_account = format!("ak-expiry:{server_hash}");

    // Check expiry
    let expiry_entry = keyring::Entry::new(SERVICE, &expiry_account)
        .map_err(|e| KeychainError::Other(e.to_string()))?;
    match expiry_entry.get_password() {
        Ok(expiry_str) => {
            let expiry: u64 = expiry_str
                .parse()
                .map_err(|_| KeychainError::Other("invalid expiry".into()))?;
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if now > expiry {
                // Expired — clean up
                let _ = delete_ak(server_hash);
                return Ok(None);
            }
        }
        Err(_) => return Ok(None),
    }

    // Load AK
    let entry = keyring::Entry::new(SERVICE, &ak_account)
        .map_err(|e| KeychainError::Other(e.to_string()))?;
    match entry.get_password() {
        Ok(ak_b64) => decode_ak(&ak_b64).map(Some),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(KeychainError::Other(e.to_string())),
    }
}

/// Delete AK and expiry from keychain.
pub fn delete_ak(server_hash: &str) -> Result<(), KeychainError> {
    let ak_account = format!("ak:{server_hash}");
    let expiry_account = format!("ak-expiry:{server_hash}");

    let entry = keyring::Entry::new(SERVICE, &ak_account)
        .map_err(|e| KeychainError::Other(e.to_string()))?;
    let _ = entry.delete_credential(); // ignore if not found

    let expiry_entry = keyring::Entry::new(SERVICE, &expiry_account)
        .map_err(|e| KeychainError::Other(e.to_string()))?;
    let _ = expiry_entry.delete_credential();

    Ok(())
}

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
}
