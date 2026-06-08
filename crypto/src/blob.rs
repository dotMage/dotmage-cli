//! Versioned blob format `v1:base64(nonce):base64(ciphertext)` (spec A.5).

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;

use crate::secret::EncryptedSecret;

const NONCE_LEN: usize = 24;

/// Encode an encrypted secret into the wire format: `v1:base64(nonce):base64(ct)`.
pub fn encode_blob(encrypted: &EncryptedSecret) -> String {
    let nonce_b64 = B64.encode(encrypted.nonce);
    let ct_b64 = B64.encode(&encrypted.ciphertext);
    format!("v1:{nonce_b64}:{ct_b64}")
}

/// Decode a wire-format blob back into nonce + ciphertext.
pub fn decode_blob(encoded: &str) -> Result<EncryptedSecret, BlobError> {
    let mut parts = encoded.splitn(3, ':');

    let version = parts.next().ok_or(BlobError::InvalidFormat)?;
    let nonce_b64 = parts.next().ok_or(BlobError::InvalidFormat)?;
    let ct_b64 = parts.next().ok_or(BlobError::InvalidFormat)?;

    match version {
        "v1" => {}
        other => {
            let v = other
                .strip_prefix('v')
                .and_then(|n| n.parse::<u8>().ok())
                .unwrap_or(0);
            return Err(BlobError::UnsupportedVersion(other.to_string(), v));
        }
    }

    let nonce_bytes = B64
        .decode(nonce_b64)
        .map_err(|e| BlobError::Base64(e.to_string()))?;

    let nonce: [u8; NONCE_LEN] = nonce_bytes
        .try_into()
        .map_err(|_| BlobError::InvalidFormat)?;

    let ciphertext = B64
        .decode(ct_b64)
        .map_err(|e| BlobError::Base64(e.to_string()))?;

    Ok(EncryptedSecret { nonce, ciphertext })
}

#[derive(Debug, thiserror::Error)]
pub enum BlobError {
    #[error("unsupported blob version '{0}' (parsed as {1})")]
    UnsupportedVersion(String, u8),
    #[error("invalid blob format")]
    InvalidFormat,
    #[error("base64 decode error: {0}")]
    Base64(String),
}
