//! `dmage diff <app>` — compare local .env with remote.

use dotmage_client::types::RevSpec;
use dotmage_crypto::blob;
use dotmage_crypto::secret;
use std::collections::BTreeSet;

use super::{CliError, Context};

pub fn run(ctx: &mut Context, name: &str, show_values: bool) -> Result<(), CliError> {
    let ak = ctx.require_ak()?;
    let env_name = ctx.active_env.clone();

    let local_path = std::path::Path::new(".env");
    if !local_path.exists() {
        return Err(CliError::Other("no local .env file".into()));
    }
    let local_data = std::fs::read(local_path)?;
    let local_vars = parse_env_map(&local_data);

    let revision = ctx
        .backend
        .pull_revision(name, &env_name, &RevSpec::Latest)?;
    let decoded = blob::decode_blob(&revision.blob).map_err(|e| CliError::Crypto(e.to_string()))?;
    let remote_data = secret::decrypt_secret(&ak, &decoded, name, &env_name, revision.rev_number)
        .map_err(|e| CliError::Crypto(e.to_string()))?;
    let remote_vars = parse_env_map(&remote_data);

    println!("Comparing ./.env <> rev {}:", revision.rev_number);

    let all_keys: BTreeSet<&str> = local_vars
        .keys()
        .chain(remote_vars.keys())
        .map(|s| s.as_str())
        .collect();

    let mut changes = 0;
    for key in all_keys {
        let local_val = local_vars.get(key);
        let remote_val = remote_vars.get(key);

        match (local_val, remote_val) {
            (Some(l), Some(r)) if l != r => {
                if show_values {
                    println!("  ~ {key}  local={l}  remote={r}");
                } else {
                    println!("  ~ {key}   (changed)");
                }
                changes += 1;
            }
            (Some(_), None) => {
                println!("  + {key}   (local only)");
                changes += 1;
            }
            (None, Some(_)) => {
                println!("  - {key}   (remote only)");
                changes += 1;
            }
            _ => {} // identical
        }
    }

    if changes == 0 {
        println!("  (identical)");
    } else if !show_values {
        println!("(values hidden; use --show-values to reveal locally)");
    }

    Ok(())
}

fn parse_env_map(data: &[u8]) -> std::collections::BTreeMap<String, String> {
    String::from_utf8_lossy(data)
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, val) = line.split_once('=')?;
            Some((
                key.trim().to_string(),
                val.trim_matches('"').trim_matches('\'').to_string(),
            ))
        })
        .collect()
}
