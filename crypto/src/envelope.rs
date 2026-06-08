//! Authenticated key (AK) wrap / unwrap operations.

use crate::kdf::MasterKey;

/// A wrapped (encrypted) application key.
#[derive(Clone)]
pub struct WrappedKey(pub Vec<u8>);

/// Wrap an application key with the master key.
pub fn wrap_key(_master_key: &MasterKey, _app_key: &[u8; 32]) -> Result<WrappedKey, EnvelopeError> {
    todo!("AK wrap with ChaCha20-Poly1305")
}

/// Unwrap an application key using the master key.
pub fn unwrap_key(
    _master_key: &MasterKey,
    _wrapped: &WrappedKey,
) -> Result<[u8; 32], EnvelopeError> {
    todo!("AK unwrap with ChaCha20-Poly1305")
}

/// Errors that can occur during envelope operations.
#[derive(Debug, thiserror::Error)]
pub enum EnvelopeError {
    #[error("encryption failed: {0}")]
    Encryption(String),
    #[error("decryption failed: {0}")]
    Decryption(String),
}
