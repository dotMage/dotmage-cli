//! Encrypt and decrypt individual secret values.

/// Encrypt a plaintext secret value with the given application key.
pub fn encrypt_secret(_app_key: &[u8; 32], _plaintext: &[u8]) -> Result<Vec<u8>, SecretError> {
    todo!("encrypt secret with ChaCha20-Poly1305")
}

/// Decrypt a ciphertext secret value with the given application key.
pub fn decrypt_secret(_app_key: &[u8; 32], _ciphertext: &[u8]) -> Result<Vec<u8>, SecretError> {
    todo!("decrypt secret with ChaCha20-Poly1305")
}

/// Errors that can occur during secret encryption/decryption.
#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("encryption failed: {0}")]
    Encryption(String),
    #[error("decryption failed: {0}")]
    Decryption(String),
    #[error("invalid ciphertext")]
    InvalidCiphertext,
}
