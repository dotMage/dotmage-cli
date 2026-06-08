//! Filesystem-based storage backend.

use crate::backend::{Backend, BackendError};
use std::path::PathBuf;

/// A backend that reads/writes encrypted files on the local filesystem.
pub struct FsBackend {
    /// Root directory for stored envelopes.
    pub root: PathBuf,
}

impl FsBackend {
    /// Create a new `FsBackend` rooted at the given directory.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl Backend for FsBackend {
    fn fetch(&self, _app: &str, _env: &str) -> Result<Vec<u8>, BackendError> {
        todo!("FsBackend::fetch")
    }

    fn store(&self, _app: &str, _env: &str, _data: &[u8]) -> Result<(), BackendError> {
        todo!("FsBackend::store")
    }
}
