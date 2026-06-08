//! Shared data types matching the API contract (Appendix B).

use serde::{Deserialize, Serialize};

/// Account cryptographic keys (B.2, B.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountKeys {
    pub salt: String,
    pub argon_params: ArgonParamsDto,
    pub nonce_ak: String,
    pub wrapped_ak: String,
    // Recovery (Appendix J)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt_rc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce_rc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrapped_ak_rc: Option<String>,
}

/// Argon2 parameters DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgonParamsDto {
    pub memory: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub version: u32,
}

/// Account init request (B.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInitReq {
    pub salt: String,
    pub argon_params: ArgonParamsDto,
    pub nonce_ak: String,
    pub wrapped_ak: String,
    pub device_name: String,
    pub bootstrap_secret: String,
    // Recovery
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt_rc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce_rc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrapped_ak_rc: Option<String>,
}

/// Account init response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInitResp {
    pub account_id: String,
    pub device_token: String,
    pub refresh_token: String,
    pub expires_at: String,
}

/// App info (B.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub environments: Vec<String>,
    pub updated_at: String,
}

/// Environment info (B.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvInfo {
    pub name: String,
    pub latest_rev: u64,
    pub updated_at: String,
}

/// Revision metadata (for history listings — no blob).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionMeta {
    pub rev_number: u64,
    pub content_hash: Option<String>,
    pub created_at: String,
    pub device_id: String,
    pub rollback_of: Option<u64>,
}

/// Full revision with blob (for pull).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    pub rev_number: u64,
    pub blob: String,
    pub content_hash: Option<String>,
    pub created_at: String,
    pub device_id: String,
    pub parent_rev: Option<u64>,
    pub rollback_of: Option<u64>,
}

/// Which revision to pull.
#[derive(Debug, Clone)]
pub enum RevSpec {
    Latest,
    Number(u64),
}

/// Health info (B.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthInfo {
    pub status: String,
    pub version: String,
    pub account_exists: bool,
}
