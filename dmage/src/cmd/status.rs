//! `dmage status` — show sync status.

use super::{CliError, Context};

pub fn run(ctx: &Context) -> Result<(), CliError> {
    let apps = ctx.backend.list_apps()?;

    if apps.is_empty() {
        ctx.print("no apps");
        return Ok(());
    }

    println!("{:<16} {:<8} {:<10} {}", "APP", "ENV", "LATEST", "UPDATED");
    for app in &apps {
        let envs = ctx.backend.list_envs(&app.name)?;
        for env in &envs {
            println!(
                "{:<16} {:<8} rev {:<6} {}",
                app.name,
                env.name,
                env.latest_rev,
                if env.updated_at.is_empty() {
                    "-"
                } else {
                    &env.updated_at[..std::cmp::min(19, env.updated_at.len())]
                }
            );
        }
    }
    Ok(())
}
