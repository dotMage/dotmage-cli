//! HTTP-based storage backend — Phase 4.

use crate::backend::{Backend, BackendError};
use crate::types::*;

/// A backend that communicates with the dotMage server over HTTP.
pub struct HttpBackend {
    pub base_url: String,
}

impl HttpBackend {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

impl Backend for HttpBackend {
    fn account_exists(&self) -> Result<bool, BackendError> { todo!("Phase 4") }
    fn account_init(&self, _req: &AccountInitReq) -> Result<AccountInitResp, BackendError> { todo!("Phase 4") }
    fn get_account_keys(&self) -> Result<AccountKeys, BackendError> { todo!("Phase 4") }
    fn update_account_keys(&self, _keys: &AccountKeys) -> Result<(), BackendError> { todo!("Phase 4") }
    fn list_apps(&self) -> Result<Vec<AppInfo>, BackendError> { todo!("Phase 4") }
    fn create_app(&self, _name: &str) -> Result<(), BackendError> { todo!("Phase 4") }
    fn list_envs(&self, _app: &str) -> Result<Vec<EnvInfo>, BackendError> { todo!("Phase 4") }
    fn create_env(&self, _app: &str, _env: &str, _copy_from: Option<&str>) -> Result<(), BackendError> { todo!("Phase 4") }
    fn delete_env(&self, _app: &str, _env: &str) -> Result<(), BackendError> { todo!("Phase 4") }
    fn push_revision(&self, _app: &str, _env: &str, _blob: &str, _parent_rev: u64) -> Result<RevisionMeta, BackendError> { todo!("Phase 4") }
    fn pull_revision(&self, _app: &str, _env: &str, _rev: &RevSpec) -> Result<Revision, BackendError> { todo!("Phase 4") }
    fn list_revisions(&self, _app: &str, _env: &str) -> Result<Vec<RevisionMeta>, BackendError> { todo!("Phase 4") }
    fn rollback(&self, _app: &str, _env: &str, _to_rev: u64) -> Result<RevisionMeta, BackendError> { todo!("Phase 4") }
}
