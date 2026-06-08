//! `dmage exec <app> -- <command>` — run with secrets in memory, no disk write.

use dotmage_client::types::RevSpec;
use dotmage_crypto::blob;
use dotmage_crypto::secret;

use super::{CliError, Context};

pub fn run(ctx: &mut Context, name: &str, command: &[String]) -> Result<(), CliError> {
    if command.is_empty() {
        return Err(CliError::Other("no command specified after --".into()));
    }

    let ak = ctx.require_ak()?;
    let env_name = ctx.active_env.clone();

    let revision = ctx
        .backend
        .pull_revision(name, &env_name, &RevSpec::Latest)?;
    let decoded = blob::decode_blob(&revision.blob).map_err(|e| CliError::Crypto(e.to_string()))?;
    let plaintext = secret::decrypt_secret(&ak, &decoded, name, &env_name, revision.rev_number)
        .map_err(|e| CliError::Crypto(e.to_string()))?;

    // Parse .env content into key-value pairs
    let env_vars = parse_env(&plaintext);

    // Run the command with injected env vars
    let status = std::process::Command::new(&command[0])
        .args(&command[1..])
        .envs(env_vars)
        .status()?;

    std::process::exit(status.code().unwrap_or(1));
}

fn parse_env(data: &[u8]) -> Vec<(String, String)> {
    String::from_utf8_lossy(data)
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, val) = line.split_once('=')?;
            let val = val.trim_matches('"').trim_matches('\'');
            Some((key.trim().to_string(), val.to_string()))
        })
        .collect()
}
