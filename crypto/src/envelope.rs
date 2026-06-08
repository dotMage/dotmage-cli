//! Account Key (AK) wrap/unwrap using XChaCha20-Poly1305 (spec A.2, A.3).

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::XChaCha20Poly1305;
use rand::RngCore;
use zeroize::Zeroizing;

use crate::kdf::MasterKey;

/// AAD for AK envelope (spec A.3).
const AK_AAD: &[u8] = b"dotmage-ak-v1";
const NONCE_LEN: usize = 24;
pub const AK_LEN: usize = 32;

/// Result of wrapping AK: nonce + ciphertext.
#[derive(Clone, Debug)]
pub struct WrappedAk {
    pub nonce: [u8; NONCE_LEN],
    pub ciphertext: Vec<u8>,
}

/// Generate a random 32-byte Account Key (spec A.2).
pub fn generate_account_key() -> Zeroizing<[u8; AK_LEN]> {
    let mut ak = Zeroizing::new([0u8; AK_LEN]);
    rand::rngs::OsRng.fill_bytes(&mut *ak);
    ak
}

/// Wrap AK with MK using XChaCha20-Poly1305 + AAD (spec A.3).
pub fn wrap_ak(mk: &MasterKey, ak: &[u8; AK_LEN]) -> Result<WrappedAk, EnvelopeError> {
    let cipher = XChaCha20Poly1305::new(mk.as_bytes().into());

    let mut nonce = [0u8; NONCE_LEN];
    rand::rngs::OsRng.fill_bytes(&mut nonce);

    let payload = Payload {
        msg: ak.as_slice(),
        aad: AK_AAD,
    };

    let ciphertext = cipher
        .encrypt((&nonce).into(), payload)
        .map_err(|e| EnvelopeError::Encryption(e.to_string()))?;

    Ok(WrappedAk { nonce, ciphertext })
}

/// Unwrap AK from its envelope using MK (spec A.3).
pub fn unwrap_ak(
    mk: &MasterKey,
    wrapped: &WrappedAk,
) -> Result<Zeroizing<[u8; AK_LEN]>, EnvelopeError> {
    let cipher = XChaCha20Poly1305::new(mk.as_bytes().into());

    let payload = Payload {
        msg: wrapped.ciphertext.as_slice(),
        aad: AK_AAD,
    };

    let plaintext = cipher
        .decrypt((&wrapped.nonce).into(), payload)
        .map_err(|_| {
            EnvelopeError::Decryption(
                "AEAD authentication failed (wrong password or tampered data)".into(),
            )
        })?;

    let ak: [u8; AK_LEN] = plaintext
        .try_into()
        .map_err(|_| EnvelopeError::Decryption("unexpected AK length".into()))?;

    Ok(Zeroizing::new(ak))
}

/// Wrap AK with a recovery key (spec Appendix J).
pub fn wrap_ak_recovery(rk: &[u8; AK_LEN], ak: &[u8; AK_LEN]) -> Result<WrappedAk, EnvelopeError> {
    let cipher = XChaCha20Poly1305::new(rk.into());

    let mut nonce = [0u8; NONCE_LEN];
    rand::rngs::OsRng.fill_bytes(&mut nonce);

    let payload = Payload {
        msg: ak.as_slice(),
        aad: b"dotmage-ak-rc-v1",
    };

    let ciphertext = cipher
        .encrypt((&nonce).into(), payload)
        .map_err(|e| EnvelopeError::Encryption(e.to_string()))?;

    Ok(WrappedAk { nonce, ciphertext })
}

/// Unwrap AK using a recovery key (spec Appendix J).
pub fn unwrap_ak_recovery(
    rk: &[u8; AK_LEN],
    wrapped: &WrappedAk,
) -> Result<Zeroizing<[u8; AK_LEN]>, EnvelopeError> {
    let cipher = XChaCha20Poly1305::new(rk.into());

    let payload = Payload {
        msg: wrapped.ciphertext.as_slice(),
        aad: b"dotmage-ak-rc-v1",
    };

    let plaintext = cipher
        .decrypt((&wrapped.nonce).into(), payload)
        .map_err(|_| {
            EnvelopeError::Decryption(
                "AEAD authentication failed (wrong recovery code or tampered data)".into(),
            )
        })?;

    let mut ak = Zeroizing::new([0u8; AK_LEN]);
    ak.copy_from_slice(&plaintext);
    Ok(ak)
}

#[derive(Debug, thiserror::Error)]
pub enum EnvelopeError {
    #[error("encryption failed: {0}")]
    Encryption(String),
    #[error("decryption failed: {0}")]
    Decryption(String),
}
