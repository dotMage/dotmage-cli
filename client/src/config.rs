//! Configuration file (~/.config/dotmage/config.toml) handling.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Resolved dotMage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server base URL.
    #[serde(default)]
    pub server_url: Option<String>,
    /// Device name for this machine.
    #[serde(default)]
    pub device_name: Option<String>,
    /// Active environment (per-project or global default).
    #[serde(default = "default_env")]
    pub active_env: String,
    /// TTL for keychain cache in seconds (default 7 days).
    #[serde(default = "default_ttl")]
    pub key_ttl_secs: u64,
    /// List of protected environment names.
    #[serde(default = "default_protected")]
    pub protected_envs: Vec<String>,
    /// Path to FsBackend root (for local mode).
    #[serde(default)]
    pub fs_backend_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_url: None,
            device_name: None,
            active_env: default_env(),
            key_ttl_secs: default_ttl(),
            protected_envs: default_protected(),
            fs_backend_path: None,
        }
    }
}

fn default_env() -> String {
    "dev".into()
}

fn default_ttl() -> u64 {
    7 * 24 * 3600 // 7 days
}

fn default_protected() -> Vec<String> {
    vec!["prod".into(), "production".into()]
}

impl Config {
    /// Default config directory: ~/.config/dotmage/
    pub fn default_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("dotmage")
    }

    pub fn default_path() -> PathBuf {
        Self::default_dir().join("config.toml")
    }

    /// Load from default path, returning defaults if file doesn't exist.
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::default_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        Self::load_from(&path)
    }

    /// Load from a specific path.
    pub fn load_from(path: &PathBuf) -> Result<Self, ConfigError> {
        let data = std::fs::read_to_string(path)?;
        toml::from_str(&data).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    /// Save config to default path.
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = toml::to_string_pretty(self).map_err(|e| ConfigError::Parse(e.to_string()))?;
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Resolve the FsBackend root directory.
    pub fn fs_root(&self) -> PathBuf {
        self.fs_backend_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| Self::default_dir().join("local"))
    }

    /// Resolve the server identifier for keychain.
    pub fn server_id(&self) -> String {
        self.server_url.clone().unwrap_or_else(|| "local".into())
    }

    /// Check if an environment name is protected (prod-guard).
    pub fn is_protected_env(&self, env: &str) -> bool {
        self.protected_envs.iter().any(|p| p == env)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
}
