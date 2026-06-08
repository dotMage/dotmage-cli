//! Zeroize helpers for sensitive data (spec A.6).

use zeroize::Zeroize;

/// A `Vec<u8>` that zeroizes on drop — use for any decrypted plaintext.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct SecureBytes(Vec<u8>);

impl SecureBytes {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_inner(mut self) -> Vec<u8> {
        let inner = std::mem::take(&mut self.0);
        // self will drop with empty vec (zeroize is a no-op on empty)
        inner
    }
}

impl AsRef<[u8]> for SecureBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
