//! `v1:...` blob format — encoding and decoding of encrypted blobs.

/// A versioned encrypted blob.
#[derive(Clone, Debug)]
pub struct Blob {
    /// Version tag (e.g. 1).
    pub version: u8,
    /// Raw ciphertext bytes.
    pub ciphertext: Vec<u8>,
}

/// Encode a ciphertext into the `v1:<base64>` string format.
pub fn encode_blob(_ciphertext: &[u8]) -> String {
    todo!("encode v1 blob")
}

/// Decode a `v1:<base64>` string into raw ciphertext bytes.
pub fn decode_blob(_encoded: &str) -> Result<Vec<u8>, BlobError> {
    todo!("decode v1 blob")
}

/// Errors that can occur during blob encoding/decoding.
#[derive(Debug, thiserror::Error)]
pub enum BlobError {
    #[error("unsupported blob version: {0}")]
    UnsupportedVersion(u8),
    #[error("invalid blob format")]
    InvalidFormat,
    #[error("base64 decode error: {0}")]
    Base64(String),
}
