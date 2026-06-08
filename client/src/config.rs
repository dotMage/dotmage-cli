//! Configuration file (`config.toml`) handling.

use std::path::PathBuf;

/// Resolved dotMage configuration.
#[derive(Debug, Default)]
pub struct Config {
    /// Server base URL.
    pub server_url: Option<String>,
    /// Default application name.
    pub default_app: Option<String>,
}

/// Load configuration from the default path (`~/.dotmage/config.toml`).
pub fn load_config() -> Result<Config, ConfigError> {
    todo!("load config.toml")
}

/// Load configuration from a specific path.
pub fn load_config_from(_path: &PathBuf) -> Result<Config, ConfigError> {
    todo!("load config.toml from path")
}

/// Errors that can occur when loading configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
}
