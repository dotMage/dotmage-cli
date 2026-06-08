//! HTTP-based storage backend.

use crate::backend::{Backend, BackendError};

/// A backend that communicates with the dotMage server over HTTP.
pub struct HttpBackend {
    /// Base URL of the dotMage server.
    pub base_url: String,
}

impl HttpBackend {
    /// Create a new `HttpBackend` pointing at the given server URL.
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

impl Backend for HttpBackend {
    fn fetch(&self, _app: &str, _env: &str) -> Result<Vec<u8>, BackendError> {
        todo!("HttpBackend::fetch")
    }

    fn store(&self, _app: &str, _env: &str, _data: &[u8]) -> Result<(), BackendError> {
        todo!("HttpBackend::store")
    }
}
