//! OS keychain integration for storing the master password / derived key.

/// A trait for keychain backends (macOS Keychain, Linux secret-service, etc.).
pub trait KeychainBackend {
    /// Store a secret under the given service and account.
    fn set_secret(
        &self,
        service: &str,
        account: &str,
        secret: &[u8],
    ) -> Result<(), KeychainError>;

    /// Retrieve a secret for the given service and account.
    fn get_secret(
        &self,
        service: &str,
        account: &str,
    ) -> Result<Vec<u8>, KeychainError>;

    /// Delete a secret for the given service and account.
    fn delete_secret(
        &self,
        service: &str,
        account: &str,
    ) -> Result<(), KeychainError>;
}

/// Errors that can occur in keychain operations.
#[derive(Debug, thiserror::Error)]
pub enum KeychainError {
    #[error("secret not found")]
    NotFound,
    #[error("keychain error: {0}")]
    Other(String),
}
