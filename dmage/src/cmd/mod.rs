//! CLI command implementations.

pub mod apps;
pub mod auth;
pub mod diff;
pub mod env;
pub mod exec;
pub mod history;
pub mod init;
pub mod lock;
pub mod pull;
pub mod push;
pub mod rollback;
pub mod status;

use dotmage_client::backend::Backend;
use dotmage_client::backend_fs::FsBackend;
use dotmage_client::backend_http::HttpBackend;
use dotmage_client::config::Config;
use dotmage_client::keychain;
use dotmage_client::token;
use std::process::ExitCode;

/// Shared context for all commands.
pub struct Context {
    pub config: Config,
    pub backend: Box<dyn Backend>,
    pub active_env: String,
    pub quiet: bool,
    pub json: bool,
    /// Cached AK (loaded on demand).
    ak: Option<[u8; 32]>,
}

impl Context {
    pub fn load(env_override: Option<String>, quiet: bool, json: bool) -> Result<Self, CliError> {
        let config = Config::load().map_err(|e| CliError::Config(e.to_string()))?;
        let active_env = env_override.unwrap_or_else(|| config.active_env.clone());

        let backend: Box<dyn Backend> = if let Some(ref url) = config.server_url {
            // HTTP mode — connect to server
            let server_hash = keychain::server_hash(url);
            let device_token = token::load_tokens(&server_hash)
                .ok()
                .flatten()
                .map(|t| t.device_token)
                .unwrap_or_default();
            Box::new(HttpBackend::new(url, &device_token))
        } else {
            // Local mode — FsBackend
            Box::new(FsBackend::new(config.fs_root()))
        };

        Ok(Self {
            config,
            backend,
            active_env,
            quiet,
            json,
            ak: None,
        })
    }

    /// Get AK from keychain cache. Returns error code 3 if not available.
    pub fn require_ak(&mut self) -> Result<[u8; 32], CliError> {
        if let Some(ak) = self.ak {
            return Ok(ak);
        }

        let server_hash = keychain::server_hash(&self.config.server_id());
        match keychain::load_ak(&server_hash) {
            Ok(Some(ak)) => {
                self.ak = Some(ak);
                Ok(ak)
            }
            Ok(None) => Err(CliError::NotAuthenticated),
            Err(e) => Err(CliError::Keychain(e.to_string())),
        }
    }

    pub fn print(&self, msg: &str) {
        if !self.quiet {
            println!("{msg}");
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("{0}")]
    Config(String),
    #[error("not authenticated — run: dmage auth")]
    NotAuthenticated,
    #[error("keychain error: {0}")]
    Keychain(String),
    #[error("{0}")]
    Backend(#[from] dotmage_client::backend::BackendError),
    #[error("{0}")]
    Crypto(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

impl CliError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            CliError::NotAuthenticated => ExitCode::from(3),
            CliError::Backend(dotmage_client::backend::BackendError::Conflict(_)) => {
                ExitCode::from(4)
            }
            CliError::Backend(dotmage_client::backend::BackendError::NotFound(_)) => {
                ExitCode::from(1)
            }
            _ => ExitCode::from(1),
        }
    }
}
