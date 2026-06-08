//! `dmage push <app>` — encrypt local .env → new revision.

use dotmage_client::types::RevSpec;
use dotmage_crypto::blob;
use dotmage_crypto::secret;

use super::{CliError, Context};

pub fn run(ctx: &mut Context, name: &str, file: &str) -> Result<(), CliError> {
    let ak = ctx.require_ak()?;
    let env_name = ctx.active_env.clone();

    // Prod-guard
    if ctx.config.is_protected_env(&env_name) {
        eprint!("This will push to PROTECTED env '{env_name}'. Type '{env_name}' to confirm: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim() != env_name {
            return Err(CliError::Other("aborted".into()));
        }
    }

    let path = std::path::Path::new(file);
    if !path.exists() {
        return Err(CliError::Other(format!("no {file} found")));
    }

    let plaintext = std::fs::read(path)?;

    // Get current latest rev
    let envs = ctx.backend.list_envs(name)?;
    let env_info = envs.iter().find(|e| e.name == env_name);
    let parent_rev = env_info.map(|e| e.latest_rev).unwrap_or(0);

    // Check if content is identical to latest
    if parent_rev > 0 {
        let latest = ctx
            .backend
            .pull_revision(name, &env_name, &RevSpec::Latest)?;
        let decoded =
            blob::decode_blob(&latest.blob).map_err(|e| CliError::Crypto(e.to_string()))?;
        if let Ok(prev_plaintext) =
            secret::decrypt_secret(&ak, &decoded, name, &env_name, latest.rev_number)
        {
            if prev_plaintext == plaintext {
                ctx.print(&format!("nothing to push (identical to rev {parent_rev})"));
                return Ok(());
            }
        }
    }

    let new_rev = parent_rev + 1;
    let encrypted = secret::encrypt_secret(&ak, &plaintext, name, &env_name, new_rev)
        .map_err(|e| CliError::Crypto(e.to_string()))?;
    let blob_str = blob::encode_blob(&encrypted);

    let meta = ctx
        .backend
        .push_revision(name, &env_name, &blob_str, parent_rev)?;

    let key_count = count_env_keys(&plaintext);
    ctx.print(&format!(
        "Pushed revision {} ({key_count} keys).",
        meta.rev_number
    ));
    Ok(())
}

fn count_env_keys(data: &[u8]) -> usize {
    String::from_utf8_lossy(data)
        .lines()
        .filter(|l| {
            let l = l.trim();
            !l.is_empty() && !l.starts_with('#') && l.contains('=')
        })
        .count()
}
