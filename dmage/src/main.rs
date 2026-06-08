//! dmage — the dotMage CLI entry point.

use clap::{Parser, Subcommand};

mod cmd;

#[derive(Parser)]
#[command(name = "dmage", about = "dotMage — E2E-encrypted .env secret manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with the dotMage server.
    Auth,
    /// Initialize a new dotMage project in the current directory.
    Init,
    /// Push local secrets to the server.
    Push,
    /// Pull secrets from the server.
    Pull,
    /// Execute a command with secrets injected into the environment.
    Exec,
    /// Show diff between local and remote secrets.
    Diff,
    /// Show secret change history.
    History,
    /// Rollback secrets to a previous version.
    Rollback,
    /// Manage applications.
    Apps,
    /// Show current sync status.
    Status,
    /// Lock or unlock an environment.
    Lock,
    /// Log out and clear stored credentials.
    Logout,
    /// Generate a device token for CI/CD.
    GenToken,
    /// Print resolved environment variables (for debugging).
    Env,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth => println!("auth: not implemented"),
        Commands::Init => println!("init: not implemented"),
        Commands::Push => println!("push: not implemented"),
        Commands::Pull => println!("pull: not implemented"),
        Commands::Exec => println!("exec: not implemented"),
        Commands::Diff => println!("diff: not implemented"),
        Commands::History => println!("history: not implemented"),
        Commands::Rollback => println!("rollback: not implemented"),
        Commands::Apps => println!("apps: not implemented"),
        Commands::Status => println!("status: not implemented"),
        Commands::Lock => println!("lock: not implemented"),
        Commands::Logout => println!("logout: not implemented"),
        Commands::GenToken => println!("gen-token: not implemented"),
        Commands::Env => println!("env: not implemented"),
    }
}
