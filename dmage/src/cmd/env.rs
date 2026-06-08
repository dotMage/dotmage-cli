//! `dmage env` — manage environments.

use super::{CliError, Context};

pub enum EnvCmd {
    List(String),
    New(String, String, Option<String>),
    Rm(String, String, bool),
}

pub fn run(ctx: &Context, action: Option<EnvCmd>) -> Result<(), CliError> {
    match action {
        None => {
            // Show active environment
            println!("active: {}", ctx.active_env);
            Ok(())
        }
        Some(EnvCmd::List(app)) => {
            let envs = ctx.backend.list_envs(&app)?;
            if envs.is_empty() {
                ctx.print("no environments");
                return Ok(());
            }
            println!("{:<12} {:<10} {}", "NAME", "LATEST", "UPDATED");
            for env in &envs {
                let marker = if env.name == ctx.active_env { " *" } else { "" };
                println!(
                    "{:<12} rev {:<6} {}{}",
                    env.name,
                    env.latest_rev,
                    if env.updated_at.is_empty() {
                        "-"
                    } else {
                        &env.updated_at[..std::cmp::min(19, env.updated_at.len())]
                    },
                    marker,
                );
            }
            Ok(())
        }
        Some(EnvCmd::New(app, name, copy_from)) => {
            ctx.backend.create_env(&app, &name, copy_from.as_deref())?;
            ctx.print(&format!("Created environment '{name}' in app '{app}'."));
            Ok(())
        }
        Some(EnvCmd::Rm(app, name, yes)) => {
            if ctx.config.is_protected_env(&name) && !yes {
                eprint!("This will DELETE protected env '{name}'. Type '{name}' to confirm: ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim() != name {
                    return Err(CliError::Other("aborted".into()));
                }
            } else if !yes {
                eprint!("Delete env '{name}' from '{app}'? [y/N] ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    return Err(CliError::Other("aborted".into()));
                }
            }

            ctx.backend.delete_env(&app, &name)?;
            ctx.print(&format!("Deleted environment '{name}' from app '{app}'."));
            Ok(())
        }
    }
}
