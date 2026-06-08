//! `dmage clean` — wipe all local dotMage data from this device.

use dotmage_client::config::Config;
use dotmage_client::keychain;

use super::{CliError, Context};

pub fn run(ctx: &Context) -> Result<(), CliError> {
    let config_dir = Config::default_dir();

    eprint!(
        "\x1b[33m  This will delete ALL local dotMage data:\x1b[0m\n\
         \x1b[90m  - Cached keys\n\
         - Device tokens\n\
         - Config\n\
         - Local storage\x1b[0m\n\
         \n\
         Type 'yes' to confirm: "
    );

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim() != "yes" {
        println!("  aborted");
        return Ok(());
    }

    // Delete AK
    let server_hash = keychain::server_hash(&ctx.config.server_id());
    let _ = keychain::delete_ak(&server_hash);

    // Delete entire config directory
    if config_dir.exists() {
        std::fs::remove_dir_all(&config_dir)?;
    }

    println!("\x1b[32m  ✓\x1b[0m All local data removed.");
    Ok(())
}
