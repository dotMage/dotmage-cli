//! HTTP-based storage backend — talks to dotMage server API.

use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::backend::{Backend, BackendError};
use crate::types::*;

/// A backend that communicates with the dotMage server over HTTP.
pub struct HttpBackend {
    base_url: String,
    client: Client,
    token: String,
}

#[derive(Deserialize)]
struct ErrorBody {
    error: Option<ErrorDetail>,
}

#[derive(Deserialize)]
struct ErrorDetail {
    message: Option<String>,
}

impl HttpBackend {
    pub fn new(base_url: &str, device_token: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
            token: device_token.to_string(),
        }
    }

    pub fn set_token(&mut self, token: &str) {
        self.token = token.to_string();
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api/v1{}", self.base_url, path)
    }

    fn auth_header(&self) -> (&str, String) {
        ("Authorization", format!("Bearer {}", self.token))
    }

    fn extract_error(status: StatusCode, body: &str) -> BackendError {
        if let Ok(err) = serde_json::from_str::<ErrorBody>(body) {
            if let Some(detail) = err.error {
                let msg = detail.message.unwrap_or_else(|| status.to_string());
                return match status {
                    StatusCode::NOT_FOUND => BackendError::NotFound(msg),
                    StatusCode::CONFLICT => BackendError::Conflict(msg),
                    _ => BackendError::Other(msg),
                };
            }
        }
        BackendError::Other(format!("HTTP {status}: {body}"))
    }
}

impl Backend for HttpBackend {
    fn account_exists(&self) -> Result<bool, BackendError> {
        let resp = self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .map_err(|e| BackendError::Other(e.to_string()))?;
        let data: HealthInfo = resp
            .json()
            .map_err(|e| BackendError::Other(e.to_string()))?;
        Ok(data.account_exists)
    }

    fn account_init(&self, req: &AccountInitReq) -> Result<AccountInitResp, BackendError> {
        // Server expects flat argon fields, not nested argon_params
        let body = serde_json::json!({
            "salt": req.salt,
            "argon_memory": req.argon_params.memory,
            "argon_iterations": req.argon_params.iterations,
            "argon_parallelism": req.argon_params.parallelism,
            "argon_version": req.argon_params.version,
            "nonce_ak": req.nonce_ak,
            "wrapped_ak": req.wrapped_ak,
            "device_name": req.device_name,
            "bootstrap_secret": req.bootstrap_secret,
            "salt_rc": req.salt_rc,
            "nonce_rc": req.nonce_rc,
            "wrapped_ak_rc": req.wrapped_ak_rc,
        });
        let resp = self
            .client
            .post(self.url("/account/init"))
            .json(&body)
            .send()
            .map_err(|e| BackendError::Other(e.to_string()))?;

        let status = resp.status();
        let body = resp
            .text()
            .map_err(|e| BackendError::Other(e.to_string()))?;
        if !status.is_success() {
            return Err(Self::extract_error(status, &body));
        }
        serde_json::from_str(&body).map_err(|e| BackendError::Other(e.to_string()))
    }

    fn get_account_keys(&self) -> Result<AccountKeys, BackendError> {
        let (hdr, val) = self.auth_header();
        let resp = self
            .client
            .get(self.url("/account/keys"))
            .header(hdr, val)
            .send()
            .map_err(|e| BackendError::Other(e.to_string()))?;

        let status = resp.status();
        if status == StatusCode::NOT_FOUND {
            return Err(BackendError::NotInitialized);
        }
        let body = resp
            .text()
            .map_err(|e| BackendError::Other(e.to_string()))?;
        if !status.is_success() {
            return Err(Self::extract_error(status, &body));
        }

        // Server returns flat argon fields; map to nested struct
        #[derive(Deserialize)]
        struct FlatKeys {
            salt: String,
            argon_memory: u32,
            argon_iterations: u32,
            argon_parallelism: u32,
            argon_version: u32,
            nonce_ak: String,
            wrapped_ak: String,
            salt_rc: Option<String>,
            nonce_rc: Option<String>,
            wrapped_ak_rc: Option<String>,
        }

        let flat: FlatKeys =
            serde_json::from_str(&body).map_err(|e| BackendError::Other(e.to_string()))?;
        Ok(AccountKeys {
            salt: flat.salt,
            argon_params: ArgonParamsDto {
                memory: flat.argon_memory,
                iterations: flat.argon_iterations,
                parallelism: flat.argon_parallelism,
                version: flat.argon_version,
            },
            nonce_ak: flat.nonce_ak,
            wrapped_ak: flat.wrapped_ak,
            salt_rc: flat.salt_rc,
            nonce_rc: flat.nonce_rc,
            wrapped_ak_rc: flat.wrapped_ak_rc,
        })
    }

    fn update_account_keys(&self, keys: &AccountKeys) -> Result<(), BackendError> {
        let (hdr, val) = self.auth_header();
        let resp = self
            .client
            .patch(self.url("/account/keys"))
            .header(hdr, val)
            .json(keys)
            .send()
            .map_err(|e| BackendError::Other(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp
                .text()
                .map_err(|e| BackendError::Other(e.to_string()))?;
            return Err(Self::extract_error(status, &body));
        }
        Ok(())
    }

    fn list_apps(&self) -> Result<Vec<AppInfo>, BackendError> {
        let (hdr, val) = self.auth_header();
        let resp = self
            .client
            .get(self.url("/apps"))
            .header(hdr, val)
            .send()
            .map_err(|e| BackendError::Other(e.to_string()))?;

        let status = resp.status();
        let body = resp
            .text()
            .map_err(|e| BackendError::Other(e.to_string()))?;
        if !status.is_success() {
            return Err(Self::extract_error(status, &body));
        }

        #[derive(Deserialize)]
        struct AppsResp {
            apps: Vec<AppRaw>,
        }
        #[derive(Deserialize)]
        struct AppRaw {
            name: String,
            environments: Option<Vec<EnvRaw>>,
            updated_at: Option<String>,
        }
        #[derive(Deserialize)]
        struct EnvRaw {
            name: String,
        }

        let parsed: AppsResp =
            serde_json::from_str(&body).map_err(|e| BackendError::Other(e.to_string()))?;
        Ok(parsed
            .apps
            .into_iter()
            .map(|a| AppInfo {
                name: a.name,
                environments: a
                    .environments
                    .unwrap_or_default()
                    .into_iter()
                    .map(|e| e.name)
                    .collect(),
                updated_at: a.updated_at.unwrap_or_default(),
            })
            .collect())
    }

    fn create_app(&self, name: &str) -> Result<(), BackendError> {
        let (hdr, val) = self.auth_header();
        let resp = self
            .client
            .post(self.url("/apps"))
            .header(hdr, val)
            .json(&serde_json::json!({"name": name}))
            .send()
            .map_err(|e| BackendError::Other(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp
                .text()
                .map_err(|e| BackendError::Other(e.to_string()))?;
            return Err(Self::extract_error(status, &body));
        }
        Ok(())
    }

    fn list_envs(&self, app: &str) -> Result<Vec<EnvInfo>, BackendError> {
        let (hdr, val) = self.auth_header();
        let resp = self
            .client
            .get(self.url(&format!("/apps/{app}/envs")))
            .header(hdr, val)
            .send()
            .map_err(|e| BackendError::Other(e.to_string()))?;

        let status = resp.status();
        let body = resp
            .text()
            .map_err(|e| BackendError::Other(e.to_string()))?;
        if !status.is_success() {
            return Err(Self::extract_error(status, &body));
        }

        #[derive(Deserialize)]
        struct Resp {
            environments: Vec<EnvRaw>,
        }
        #[derive(Deserialize)]
        struct EnvRaw {
            name: String,
            latest_rev: u64,
            updated_at: Option<String>,
        }

        let parsed: Resp =
            serde_json::from_str(&body).map_err(|e| BackendError::Other(e.to_string()))?;
        Ok(parsed
            .environments
            .into_iter()
            .map(|e| EnvInfo {
                name: e.name,
                latest_rev: e.latest_rev,
                updated_at: e.updated_at.unwrap_or_default(),
            })
            .collect())
    }

    fn create_env(
        &self,
        app: &str,
        env: &str,
        copy_from: Option<&str>,
    ) -> Result<(), BackendError> {
        let (hdr, val) = self.auth_header();
        let mut body = serde_json::json!({"name": env});
        if let Some(src) = copy_from {
            body["copy_from"] = serde_json::Value::String(src.to_string());
        }
        let resp = self
            .client
            .post(self.url(&format!("/apps/{app}/envs")))
            .header(hdr, val)
            .json(&body)
            .send()
            .map_err(|e| BackendError::Other(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp
                .text()
                .map_err(|e| BackendError::Other(e.to_string()))?;
            return Err(Self::extract_error(status, &body));
        }
        Ok(())
    }

    fn delete_env(&self, app: &str, env: &str) -> Result<(), BackendError> {
        let (hdr, val) = self.auth_header();
        let resp = self
            .client
            .delete(self.url(&format!("/apps/{app}/envs/{env}")))
            .header(hdr, val)
            .send()
            .map_err(|e| BackendError::Other(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp
                .text()
                .map_err(|e| BackendError::Other(e.to_string()))?;
            return Err(Self::extract_error(status, &body));
        }
        Ok(())
    }

    fn push_revision(
        &self,
        app: &str,
        env: &str,
        blob: &str,
        parent_rev: u64,
    ) -> Result<RevisionMeta, BackendError> {
        let (hdr, val) = self.auth_header();
        let resp = self
            .client
            .post(self.url(&format!("/apps/{app}/envs/{env}/revisions")))
            .header(hdr, val)
            .json(&serde_json::json!({
                "blob": blob,
                "parent_rev": parent_rev,
                "content_hash": null
            }))
            .send()
            .map_err(|e| BackendError::Other(e.to_string()))?;

        let status = resp.status();
        let body = resp
            .text()
            .map_err(|e| BackendError::Other(e.to_string()))?;
        if !status.is_success() {
            return Err(Self::extract_error(status, &body));
        }

        #[derive(Deserialize)]
        struct Resp {
            rev_number: u64,
            created_at: String,
            device_id: String,
        }
        let parsed: Resp =
            serde_json::from_str(&body).map_err(|e| BackendError::Other(e.to_string()))?;
        Ok(RevisionMeta {
            rev_number: parsed.rev_number,
            content_hash: None,
            created_at: parsed.created_at,
            device_id: parsed.device_id,
            rollback_of: None,
        })
    }

    fn pull_revision(&self, app: &str, env: &str, rev: &RevSpec) -> Result<Revision, BackendError> {
        let rev_str = match rev {
            RevSpec::Latest => "last".to_string(),
            RevSpec::Number(n) => n.to_string(),
        };
        let (hdr, val) = self.auth_header();
        let resp = self
            .client
            .get(self.url(&format!("/apps/{app}/envs/{env}/revisions/{rev_str}")))
            .header(hdr, val)
            .send()
            .map_err(|e| BackendError::Other(e.to_string()))?;

        let status = resp.status();
        let body = resp
            .text()
            .map_err(|e| BackendError::Other(e.to_string()))?;
        if !status.is_success() {
            return Err(Self::extract_error(status, &body));
        }
        serde_json::from_str(&body).map_err(|e| BackendError::Other(e.to_string()))
    }

    fn list_revisions(&self, app: &str, env: &str) -> Result<Vec<RevisionMeta>, BackendError> {
        let (hdr, val) = self.auth_header();
        let resp = self
            .client
            .get(self.url(&format!("/apps/{app}/envs/{env}/revisions")))
            .header(hdr, val)
            .send()
            .map_err(|e| BackendError::Other(e.to_string()))?;

        let status = resp.status();
        let body = resp
            .text()
            .map_err(|e| BackendError::Other(e.to_string()))?;
        if !status.is_success() {
            return Err(Self::extract_error(status, &body));
        }

        #[derive(Deserialize)]
        struct Resp {
            revisions: Vec<RevisionMeta>,
        }
        let parsed: Resp =
            serde_json::from_str(&body).map_err(|e| BackendError::Other(e.to_string()))?;
        Ok(parsed.revisions)
    }

    fn rollback(&self, app: &str, env: &str, to_rev: u64) -> Result<RevisionMeta, BackendError> {
        let (hdr, val) = self.auth_header();
        let resp = self
            .client
            .post(self.url(&format!("/apps/{app}/envs/{env}/rollback")))
            .header(hdr, val)
            .json(&serde_json::json!({"to_rev": to_rev}))
            .send()
            .map_err(|e| BackendError::Other(e.to_string()))?;

        let status = resp.status();
        let body = resp
            .text()
            .map_err(|e| BackendError::Other(e.to_string()))?;
        if !status.is_success() {
            return Err(Self::extract_error(status, &body));
        }

        #[derive(Deserialize)]
        struct Resp {
            rev_number: u64,
            copied_from: u64,
        }
        let parsed: Resp =
            serde_json::from_str(&body).map_err(|e| BackendError::Other(e.to_string()))?;
        Ok(RevisionMeta {
            rev_number: parsed.rev_number,
            content_hash: None,
            created_at: String::new(),
            device_id: String::new(),
            rollback_of: Some(parsed.copied_from),
        })
    }
}
