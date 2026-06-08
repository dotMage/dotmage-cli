//! Device token storage and retrieval.

/// Store a device token for the current machine.
pub fn store_device_token(_token: &str) -> Result<(), TokenError> {
    todo!("store device token")
}

/// Load the device token for the current machine.
pub fn load_device_token() -> Result<String, TokenError> {
    todo!("load device token")
}

/// Delete the stored device token.
pub fn delete_device_token() -> Result<(), TokenError> {
    todo!("delete device token")
}

/// Errors that can occur in token operations.
#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("token not found")]
    NotFound,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
