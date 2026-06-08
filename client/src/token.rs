//! Device token + refresh token persistence.
//!
//! Stored in `~/.config/dotmage/tokens.json`, keyed by server URL hash.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TokenStore {
    /// server_hash → tokens
    servers: HashMap<String, ServerTokens>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTokens {
    pub device_token: String,
    pub refresh_token: String,
    pub device_id: String,
}

fn store_path() -> PathBuf {
    Config::default_dir().join("tokens.json")
}

pub fn save_tokens(server_hash: &str, tokens: &ServerTokens) -> Result<(), TokenError> {
    let mut store = load_store()?;
    store
        .servers
        .insert(server_hash.to_string(), tokens.clone());
    let path = store_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data =
        serde_json::to_string_pretty(&store).map_err(|e| TokenError::Other(e.to_string()))?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn load_tokens(server_hash: &str) -> Result<Option<ServerTokens>, TokenError> {
    let store = load_store()?;
    Ok(store.servers.get(server_hash).cloned())
}

pub fn delete_tokens(server_hash: &str) -> Result<(), TokenError> {
    let mut store = load_store()?;
    store.servers.remove(server_hash);
    let path = store_path();
    if path.exists() {
        let data =
            serde_json::to_string_pretty(&store).map_err(|e| TokenError::Other(e.to_string()))?;
        std::fs::write(path, data)?;
    }
    Ok(())
}

fn load_store() -> Result<TokenStore, TokenError> {
    let path = store_path();
    if !path.exists() {
        return Ok(TokenStore::default());
    }
    let data = std::fs::read_to_string(&path)?;
    serde_json::from_str(&data).map_err(|e| TokenError::Other(e.to_string()))
}

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}
