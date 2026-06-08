//! dmage — the dotMage CLI entry point.

use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod cmd;

#[derive(Parser)]
#[command(
    name = "dmage",
    version,
    about = "dotMage — E2E-encrypted .env secret manager"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

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
    /// Execute a command with secrets in the environment (no disk write).
    Exec {
        /// Application name.
        name: String,
        /// Command and arguments.
        #[arg(last = true)]
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
    /// Remove AK from keychain (keep device token).
    Lock,
    /// Remove AK and device token (full logout).
    Logout,
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
    let result = run(cli);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            e.exit_code()
        }
    }
}

fn run(cli: Cli) -> Result<(), cmd::CliError> {
    let mut ctx = cmd::Context::load(cli.env, cli.quiet, cli.json)?;

    match cli.command {
        Commands::Auth { server, ttl, .. } => cmd::auth::run(&mut ctx, server, ttl),
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
    }
}
