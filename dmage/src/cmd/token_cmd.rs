//! `dmage token` — print current device token.

use dotmage_client::keychain;
use dotmage_client::token;

use super::{CliError, Context};

pub fn run(ctx: &Context) -> Result<(), CliError> {
    let server_hash = keychain::server_hash(&ctx.config.server_id());
    let tokens = token::load_tokens(&server_hash)
        .map_err(|e| CliError::Other(e.to_string()))?
        .ok_or(CliError::NotAuthenticated)?;

    println!("{}", tokens.device_token);
    Ok(())
}
