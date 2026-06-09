//! dmage — the dotMage CLI entry point.

use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod cmd;

#[derive(Parser)]
#[command(
    name = "dmage",
    version,
    about = "dotMage — E2E-encrypted .env secret manager",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Override the active environment.
    #[arg(long, global = true)]
    env: Option<String>,

    /// Suppress non-error output.
    #[arg(short, long, global = true)]
    quiet: bool,

    /// JSON output for scripting.
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with the dotMage server.
    Auth {
        /// Server URL (first time only).
        #[arg(long)]
        server: Option<String>,
        /// Enrollment token (for subsequent devices).
        #[arg(long)]
        enroll: Option<String>,
        /// Cache TTL (e.g., "7d", "30d").
        #[arg(long)]
        ttl: Option<String>,
    },
    /// Initialize a new app from the current .env file.
    Init {
        /// Application name.
        name: String,
        /// Path to .env file (default: ./.env).
        #[arg(long, default_value = ".env")]
        file: String,
    },
    /// Push local .env to a new revision.
    Push {
        /// Application name.
        name: String,
        /// Path to .env file (default: ./.env).
        #[arg(long, default_value = ".env")]
        file: String,
    },
    /// Pull secrets and write to .env file.
    Pull {
        /// Application name.
        name: String,
        /// Specific revision (default: latest).
        #[arg(long)]
        rev: Option<String>,
        /// Output file path.
        #[arg(long)]
        output: Option<String>,
        /// Write to stdout instead of file.
        #[arg(long)]
        stdout: bool,
        /// Overwrite without confirmation.
        #[arg(long)]
        force: bool,
    },
    /// Run a command with secrets injected (e.g., dmage exec myapp npm dev).
    Exec {
        /// Application name.
        name: String,
        /// Command and arguments (no -- needed).
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },
    /// Show diff between local and remote.
    Diff {
        /// Application name.
        name: String,
        /// Show actual values (locally only).
        #[arg(long)]
        show_values: bool,
    },
    /// Show revision history.
    History {
        /// Application name.
        name: String,
    },
    /// Rollback to a previous revision.
    Rollback {
        /// Application name.
        name: String,
        /// Target revision number.
        #[arg(long)]
        rev: u64,
    },
    /// List all applications.
    Apps,
    /// Show current device token (for web admin login).
    Token,
    /// Show sync status.
    Status,
    /// Remove cached key (keep device token).
    Lock,
    /// Full logout (key + tokens + local data).
    Logout,
    /// Wipe all local dotMage data from this device.
    Clean,
    /// Generate enrollment/CI token.
    GenToken {
        /// Token name.
        #[arg(long)]
        name: Option<String>,
        /// TTL (e.g., "24h").
        #[arg(long, default_value = "24h")]
        ttl: String,
    },
    /// Manage environments.
    Env {
        #[command(subcommand)]
        action: Option<EnvAction>,
    },
    /// Show help.
    Help,
}

#[derive(Subcommand)]
enum EnvAction {
    /// List all environments for the active app.
    List {
        /// Application name.
        name: String,
    },
    /// Create a new environment.
    New {
        /// Application name.
        app: String,
        /// Environment name.
        name: String,
        /// Copy from existing environment.
        #[arg(long)]
        copy_from: Option<String>,
    },
    /// Delete an environment.
    Rm {
        /// Application name.
        app: String,
        /// Environment name.
        name: String,
        /// Skip confirmation.
        #[arg(long)]
        yes: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.command.is_none() {
        print_banner();
        return ExitCode::SUCCESS;
    }

    let result = run(cli);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("\x1b[31m  error:\x1b[0m {e}");
            e.exit_code()
        }
    }
}

fn run(cli: Cli) -> Result<(), cmd::CliError> {
    let command = cli.command.unwrap();

    if matches!(command, Commands::Help) {
        use clap::CommandFactory;
        Cli::command().print_help().ok();
        println!();
        return Ok(());
    }

    let mut ctx = cmd::Context::load(cli.env, cli.quiet, cli.json)?;

    match command {
        Commands::Auth { server, ttl, .. } => {
            // If --server provided, update config and recreate backend BEFORE auth runs
            if let Some(ref url) = server {
                ctx.set_server(url)?;
            }
            cmd::auth::run(&mut ctx, server, ttl)
        }
        Commands::Init { name, file } => cmd::init::run(&mut ctx, &name, &file),
        Commands::Push { name, file } => cmd::push::run(&mut ctx, &name, &file),
        Commands::Pull {
            name,
            rev,
            output,
            stdout,
            force,
        } => cmd::pull::run(
            &mut ctx,
            &name,
            rev.as_deref(),
            output.as_deref(),
            stdout,
            force,
        ),
        Commands::Exec { name, command } => cmd::exec::run(&mut ctx, &name, &command),
        Commands::Diff { name, show_values } => cmd::diff::run(&mut ctx, &name, show_values),
        Commands::History { name } => cmd::history::run(&ctx, &name),
        Commands::Rollback { name, rev } => cmd::rollback::run(&mut ctx, &name, rev),
        Commands::Apps => cmd::apps::run(&ctx),
        Commands::Token => cmd::token_cmd::run(&ctx),
        Commands::Status => cmd::status::run(&ctx),
        Commands::Lock => cmd::lock::run(&ctx),
        Commands::Logout => cmd::lock::run_logout(&ctx),
        Commands::Clean => cmd::clean::run(&ctx),
        Commands::GenToken { name, ttl } => cmd::gen_token::run(&ctx, name.as_deref(), &ttl),
        Commands::Env { action } => cmd::env::run(
            &ctx,
            action.map(|a| match a {
                EnvAction::List { name } => cmd::env::EnvCmd::List(name),
                EnvAction::New {
                    app,
                    name,
                    copy_from,
                } => cmd::env::EnvCmd::New(app, name, copy_from),
                EnvAction::Rm { app, name, yes } => cmd::env::EnvCmd::Rm(app, name, yes),
            }),
        ),
        Commands::Help => unreachable!(),
    }
}

fn print_banner() {
    let version = env!("CARGO_PKG_VERSION");
    println!("\x1b[36m");
    println!("      ·  dotMage  ·");
    println!("\x1b[0m");
    println!("  E2E-encrypted .env manager  v{version}");
    println!();

    // Show connection status
    let config = dotmage_client::config::Config::load().unwrap_or_default();
    if let Some(ref url) = config.server_url {
        let hash = dotmage_client::keychain::server_hash(url);
        let has_ak = dotmage_client::keychain::load_ak(&hash)
            .ok()
            .flatten()
            .is_some();
        let has_token = dotmage_client::token::load_tokens(&hash)
            .ok()
            .flatten()
            .is_some();

        println!("  server   \x1b[90m{url}\x1b[0m");
        if has_ak {
            println!("  auth     \x1b[32m● authenticated\x1b[0m");
        } else if has_token {
            println!("  auth     \x1b[33m● token saved, run: dmage auth\x1b[0m");
        } else {
            println!("  auth     \x1b[31m● not connected\x1b[0m");
        }
    } else {
        println!("  server   \x1b[90m(local mode)\x1b[0m");
    }

    println!();
    println!("  \x1b[90mRun \x1b[0mdmage help\x1b[90m for commands\x1b[0m");
    println!();
}
