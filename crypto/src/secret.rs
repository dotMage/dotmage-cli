//! Encrypt/decrypt .env secrets using XChaCha20-Poly1305 (spec A.4).

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::XChaCha20Poly1305;
use rand::RngCore;

const NONCE_LEN: usize = 24;

/// Build the AAD string per spec A.4:
/// `"dotmage-secret-v1|{app_name}|{env_name}|{rev_number}"`
pub fn build_aad(app_name: &str, env_name: &str, rev_number: u64) -> Vec<u8> {
    format!("dotmage-secret-v1|{app_name}|{env_name}|{rev_number}")
        .into_bytes()
}

/// Encrypted secret: nonce + ciphertext.
#[derive(Clone, Debug)]
pub struct EncryptedSecret {
    pub nonce: [u8; NONCE_LEN],
    pub ciphertext: Vec<u8>,
}

/// Encrypt a plaintext .env file with AK + AAD (spec A.4).
/// A fresh random nonce is generated per call.
pub fn encrypt_secret(
    ak: &[u8; 32],
    plaintext: &[u8],
    app_name: &str,
    env_name: &str,
    rev_number: u64,
) -> Result<EncryptedSecret, SecretError> {
    let cipher = XChaCha20Poly1305::new(ak.into());

    let mut nonce = [0u8; NONCE_LEN];
    rand::rngs::OsRng.fill_bytes(&mut nonce);

    let aad = build_aad(app_name, env_name, rev_number);
    let payload = Payload {
        msg: plaintext,
        aad: &aad,
    };

    let ciphertext = cipher
        .encrypt((&nonce).into(), payload)
        .map_err(|e| SecretError::Encryption(e.to_string()))?;

    Ok(EncryptedSecret { nonce, ciphertext })
}

/// Decrypt a ciphertext .env file with AK + AAD (spec A.4).
pub fn decrypt_secret(
    ak: &[u8; 32],
    encrypted: &EncryptedSecret,
    app_name: &str,
    env_name: &str,
    rev_number: u64,
) -> Result<Vec<u8>, SecretError> {
    let cipher = XChaCha20Poly1305::new(ak.into());

    let aad = build_aad(app_name, env_name, rev_number);
    let payload = Payload {
        msg: encrypted.ciphertext.as_slice(),
        aad: &aad,
    };

    cipher
        .decrypt((&encrypted.nonce).into(), payload)
        .map_err(|_| SecretError::Decryption("AEAD authentication failed".into()))
}

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("encryption failed: {0}")]
    Encryption(String),
    #[error("decryption failed: {0}")]
    Decryption(String),
}
