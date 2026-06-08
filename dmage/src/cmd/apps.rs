//! `dmage apps` — list all applications.

use super::{CliError, Context};

pub fn run(ctx: &Context) -> Result<(), CliError> {
    let apps = ctx.backend.list_apps()?;

    if apps.is_empty() {
        ctx.print("no apps");
        return Ok(());
    }

    println!("{:<16} {:<8} {}", "NAME", "ENVS", "UPDATED");
    for app in &apps {
        let updated = if app.updated_at.is_empty() {
            "-".to_string()
        } else {
            app.updated_at[..std::cmp::min(19, app.updated_at.len())].to_string()
        };
        println!("{:<16} {:<8} {}", app.name, app.environments.len(), updated);
    }
    Ok(())
}
