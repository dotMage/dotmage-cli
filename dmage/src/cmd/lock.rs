//! `dmage lock` / `dmage logout` — clear keychain.

use dotmage_client::keychain;

use super::{CliError, Context};

pub fn run(ctx: &Context) -> Result<(), CliError> {
    let server_hash = keychain::server_hash(&ctx.config.server_id());
    keychain::delete_ak(&server_hash).map_err(|e| CliError::Keychain(e.to_string()))?;
    ctx.print("Key removed from keychain.");
    Ok(())
}

pub fn run_logout(ctx: &Context) -> Result<(), CliError> {
    run(ctx)?;
    ctx.print("Logged out on this device.");
    Ok(())
}
