//! `dmage apps` — list all applications.

use super::{CliError, Context};

pub fn run(ctx: &Context) -> Result<(), CliError> {
    let mut apps = ctx.backend.list_apps()?;

    if apps.is_empty() {
        ctx.print("no apps");
        return Ok(());
    }

    apps.sort_by(|a, b| a.name.cmp(&b.name));

    let mut current_folder: Option<&str> = None;
    let mut first = true;

    for app in &apps {
        let (folder, short_name) = match app.name.rsplit_once('/') {
            Some((f, n)) => (Some(f), n),
            None => (None, app.name.as_str()),
        };

        // Print folder header if changed
        if folder != current_folder {
            if !first {
                println!();
            }
            if let Some(f) = folder {
                println!("  \x1b[36m{f}/\x1b[0m");
            }
            current_folder = folder;
        }

        let prefix = if folder.is_some() { "    " } else { "  " };
        let envs = app.environments.len();
        let updated = if app.updated_at.is_empty() {
            "-".to_string()
        } else {
            app.updated_at[..std::cmp::min(19, app.updated_at.len())].to_string()
        };
        println!("{prefix}{:<20} {envs:<3} envs   {updated}", short_name);
        first = false;
    }
    Ok(())
}
