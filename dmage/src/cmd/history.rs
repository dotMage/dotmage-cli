//! `dmage history <app>` — show revision history.

use super::{CliError, Context};

pub fn run(ctx: &Context, name: &str) -> Result<(), CliError> {
    let revs = ctx.backend.list_revisions(name, &ctx.active_env)?;

    if revs.is_empty() {
        ctx.print("no revisions");
        return Ok(());
    }

    println!("{:<5} {:<22} {:<12} NOTE", "REV", "WHEN", "DEVICE");
    for rev in &revs {
        let when = &rev.created_at[..std::cmp::min(19, rev.created_at.len())];
        let note = rev
            .rollback_of
            .map(|r| format!("rollback of {r}"))
            .unwrap_or_default();
        println!(
            "{:<5} {:<22} {:<12} {}",
            rev.rev_number, when, rev.device_id, note
        );
    }
    Ok(())
}
