//! `dmage gen-ci-token` — generate a scoped CI token for a specific app+env.

use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64URL;
use base64::Engine;
use dotmage_client::backend_http::HttpBackend;
use dotmage_client::keychain;

use super::{CliError, Context};

pub fn run(ctx: &Context, app: &str, env: &str, ttl: &str) -> Result<(), CliError> {
    let backend = ctx
        .backend
        .as_any()
        .downcast_ref::<HttpBackend>()
        .ok_or_else(|| {
            CliError::Other("gen-ci-token requires a server connection".into())
        })?;

    // Get AK from keychain
    let server_hash = keychain::server_hash(&ctx.config.server_id());
    let ak = keychain::load_ak(&server_hash)
        .map_err(|e| CliError::Keychain(e.to_string()))?
        .ok_or(CliError::NotAuthenticated)?;

    // Create scoped device on server
    let resp = backend.create_ci_token(app, env, ttl)?;

    let server_url = ctx.config.server_url.as_deref()
        .ok_or_else(|| CliError::Other("server_url not configured".into()))?;

    // Bundle device_token + AK + server URL into one opaque token
    let payload = serde_json::json!({
        "s": server_url,
        "t": resp.device_token,
        "r": resp.refresh_token,
        "k": base64::engine::general_purpose::STANDARD.encode(ak),
    });
    let blob = B64URL.encode(payload.to_string().as_bytes());
    let ci_token = format!("dmage_ci_{blob}");

    if ctx.quiet {
        println!("{ci_token}");
    } else {
        ctx.print(&format!("CI token for {app}/{env} (expires: {}):", resp.token_expires_at));
        println!("\n  {ci_token}\n");
        ctx.print("This token gives read access ONLY to this app+env.");
        ctx.print("Store it as a CI secret (e.g. DOTMAGE_CI_TOKEN).");
        ctx.print(&format!("\nIn your pipeline:\n  DOTMAGE_CI_TOKEN=<token> dmage pull {app} --env {env} --output .env"));
    }
    Ok(())
}
