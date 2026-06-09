//! `dmage token` — print current device token.

use dotmage_client::keychain;
use dotmage_client::token;

use super::{CliError, Context};

pub fn run(ctx: &Context) -> Result<(), CliError> {
    // Try server token first, then local
    if let Some(ref url) = ctx.config.server_url {
        let hash = keychain::server_hash(url);
        if let Ok(Some(t)) = token::load_tokens(&hash) {
            println!("{}", t.device_token);
            return Ok(());
        }
    }

    let hash = keychain::server_hash(&ctx.config.server_id());
    let tokens = token::load_tokens(&hash)
        .map_err(|e| CliError::Other(e.to_string()))?
        .ok_or(CliError::NotAuthenticated)?;

    println!("{}", tokens.device_token);
    Ok(())
}
