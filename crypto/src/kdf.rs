//! Key derivation using Argon2id (spec A.1).

use argon2::{Algorithm, Argon2, Params, Version};
use zeroize::{Zeroize, Zeroizing};

/// Fixed Argon2id parameters per spec A.1.
pub const ARGON2_MEMORY_KIB: u32 = 65_536; // 64 MiB
pub const ARGON2_ITERATIONS: u32 = 3;
pub const ARGON2_PARALLELISM: u32 = 1;
pub const ARGON2_VERSION: u32 = 0x13; // 19
pub const SALT_LEN: usize = 16;
pub const MK_LEN: usize = 32;

/// Stored Argon2 parameters (for future upgrades).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgonParams {
    pub memory: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub version: u32,
}

impl Default for ArgonParams {
    fn default() -> Self {
        Self {
            memory: ARGON2_MEMORY_KIB,
            iterations: ARGON2_ITERATIONS,
            parallelism: ARGON2_PARALLELISM,
            version: ARGON2_VERSION,
        }
    }
}

/// A 32-byte master key derived from a user password.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct MasterKey([u8; MK_LEN]);

impl MasterKey {
    pub fn as_bytes(&self) -> &[u8; MK_LEN] {
        &self.0
    }

    /// Construct from raw bytes (for tests only).
    pub fn from_bytes(bytes: [u8; MK_LEN]) -> Self {
        Self(bytes)
    }
}

/// Derive a 32-byte master key from password + salt using Argon2id.
pub fn derive_master_key(password: &[u8], salt: &[u8; SALT_LEN]) -> Result<MasterKey, KdfError> {
    derive_master_key_with_params(password, salt, &ArgonParams::default())
}

/// Derive MK with explicit parameters (for verifying against stored params).
pub fn derive_master_key_with_params(
    password: &[u8],
    salt: &[u8; SALT_LEN],
    params: &ArgonParams,
) -> Result<MasterKey, KdfError> {
    let argon_params = Params::new(
        params.memory,
        params.iterations,
        params.parallelism,
        Some(MK_LEN),
    )
    .map_err(|e| KdfError::Argon2(e.to_string()))?;

    let version = match params.version {
        0x13 => Version::V0x13,
        0x10 => Version::V0x10,
        v => return Err(KdfError::Argon2(format!("unsupported argon2 version: {v}"))),
    };

    let argon2 = Argon2::new(Algorithm::Argon2id, version, argon_params);

    let mut mk = Zeroizing::new([0u8; MK_LEN]);
    argon2
        .hash_password_into(password, salt, &mut *mk)
        .map_err(|e| KdfError::Argon2(e.to_string()))?;

    Ok(MasterKey(*mk))
}

/// Generate a random 16-byte salt using OS CSPRNG.
pub fn generate_salt() -> [u8; SALT_LEN] {
    use rand::RngCore;
    let mut salt = [0u8; SALT_LEN];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    salt
}

#[derive(Debug, thiserror::Error)]
pub enum KdfError {
    #[error("argon2 error: {0}")]
    Argon2(String),
}
