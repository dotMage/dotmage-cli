//! Abstract storage backend trait.

/// A storage backend that can read and write encrypted environment files.
pub trait Backend {
    /// Fetch the encrypted envelope for the given app/environment.
    fn fetch(
        &self,
        app: &str,
        env: &str,
    ) -> Result<Vec<u8>, BackendError>;

    /// Store an encrypted envelope for the given app/environment.
    fn store(
        &self,
        app: &str,
        env: &str,
        data: &[u8],
    ) -> Result<(), BackendError>;
}

/// Errors that can occur in a backend.
#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not found: {app}/{env}")]
    NotFound { app: String, env: String },
    #[error("backend error: {0}")]
    Other(String),
}
