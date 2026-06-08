//! Key derivation using Argon2id.

use zeroize::Zeroize;

/// A 32-byte master key derived from a user password.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct MasterKey([u8; 32]);

impl MasterKey {
    /// Access the raw key bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Derive a 32-byte master key from a password and salt using Argon2id.
pub fn derive_master_key(_password: &[u8], _salt: &[u8; 16]) -> Result<MasterKey, KdfError> {
    todo!("Argon2id KDF")
}

/// Generate a random 16-byte salt.
pub fn generate_salt() -> [u8; 16] {
    use rand::RngCore;
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Errors that can occur during key derivation.
#[derive(Debug, thiserror::Error)]
pub enum KdfError {
    #[error("argon2 error: {0}")]
    Argon2(String),
}
