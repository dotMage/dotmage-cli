//! `dmage token` — generate a one-time login token for the web admin.

use dotmage_client::backend_http::HttpBackend;

use super::{CliError, Context};

pub fn run(ctx: &Context) -> Result<(), CliError> {
    let backend = ctx
        .backend
        .as_any()
        .downcast_ref::<HttpBackend>()
        .ok_or_else(|| CliError::Other("token requires a server connection".into()))?;

    let (token, expires_at) = backend.gen_enroll_token("web-admin", "5m")?;

    if ctx.quiet {
        println!("{token}");
    } else {
        ctx.print("Web admin login token (one-time, 5 min):");
        println!("\n  {token}\n");
        ctx.print(&format!("Expires: {expires_at}"));
        ctx.print("Paste this token in the web admin login page.");
    }
    Ok(())
}
