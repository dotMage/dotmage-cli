//! `dmage rollback <app> --rev N` — rollback to a previous revision.

use super::{CliError, Context};

pub fn run(ctx: &mut Context, name: &str, to_rev: u64) -> Result<(), CliError> {
    let _ak = ctx.require_ak()?;
    let env_name = ctx.active_env.clone();

    // Prod-guard
    if ctx.config.is_protected_env(&env_name) {
        eprint!("This will rollback PROTECTED env '{env_name}'. Type '{env_name}' to confirm: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim() != env_name {
            return Err(CliError::Other("aborted".into()));
        }
    }

    let meta = ctx.backend.rollback(name, &env_name, to_rev)?;
    ctx.print(&format!(
        "Rolled back to rev {to_rev}. Created new revision {}.",
        meta.rev_number
    ));
    Ok(())
}
