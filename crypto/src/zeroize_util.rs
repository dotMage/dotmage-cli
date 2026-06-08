//! Zeroize helpers for sensitive data.

use zeroize::Zeroize;

/// A wrapper around a `Vec<u8>` that zeroizes on drop.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct SecureBytes(Vec<u8>);

impl SecureBytes {
    /// Create a new `SecureBytes` from a byte vector.
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// Access the inner bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
