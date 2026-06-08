//! `dmage lock` / `dmage logout` — clear keychain / tokens.

use dotmage_client::keychain;
use dotmage_client::token;

use super::{CliError, Context};

pub fn run(ctx: &Context) -> Result<(), CliError> {
    let server_hash = keychain::server_hash(&ctx.config.server_id());
    keychain::delete_ak(&server_hash).map_err(|e| CliError::Keychain(e.to_string()))?;
    ctx.print("Key removed from keychain.");
    Ok(())
}

pub fn run_logout(ctx: &Context) -> Result<(), CliError> {
    let server_hash = keychain::server_hash(&ctx.config.server_id());
    keychain::delete_ak(&server_hash).map_err(|e| CliError::Keychain(e.to_string()))?;
    token::delete_tokens(&server_hash)
        .map_err(|e: token::TokenError| CliError::Other(e.to_string()))?;
    ctx.print("Logged out on this device.");
    Ok(())
}
