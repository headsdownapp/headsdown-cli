mod auth;
mod client;
mod commands;
mod format;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io;

/// HeadsDown CLI — manage your availability from the terminal
#[derive(Parser)]
#[command(name = "hd", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// API base URL (defaults to https://headsdown.app)
    #[arg(long, global = true, env = "HEADSDOWN_API_URL")]
    api_url: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with HeadsDown via Device Flow
    Auth,

    /// Show your current availability status
    Status,

    /// Set your mode to busy
    Busy {
        /// Duration (e.g. "2h", "30m", "90min", "until 5pm")
        duration: Option<String>,
    },

    /// Set your mode to online
    Online,

    /// Set your mode to offline
    Offline,

    /// Set your mode to limited
    Limited {
        /// Duration (e.g. "2h", "30m", "90min", "until 5pm")
        duration: Option<String>,
    },

    /// Submit a task proposal and get a verdict
    Verdict {
        /// Task description
        description: String,

        /// Estimated number of files to change
        #[arg(long)]
        files: Option<i32>,

        /// Estimated minutes to complete
        #[arg(long)]
        minutes: Option<i32>,

        /// AI model being used
        #[arg(long)]
        model: Option<String>,
    },

    /// List available presets
    Presets,

    /// Activate a preset by name or ID
    Preset {
        /// Preset name or ID
        name: String,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let api_url = cli
        .api_url
        .unwrap_or_else(|| "https://headsdown.app".to_string());

    match cli.command {
        Commands::Auth => commands::auth::run(&api_url).await,
        Commands::Status => commands::status::run(&api_url).await,
        Commands::Busy { duration } => commands::mode::run(&api_url, "BUSY", duration).await,
        Commands::Online => commands::mode::run(&api_url, "ONLINE", None).await,
        Commands::Offline => commands::mode::run(&api_url, "OFFLINE", None).await,
        Commands::Limited { duration } => commands::mode::run(&api_url, "LIMITED", duration).await,
        Commands::Verdict {
            description,
            files,
            minutes,
            model,
        } => commands::verdict::run(&api_url, &description, files, minutes, model).await,
        Commands::Presets => commands::presets::run(&api_url).await,
        Commands::Preset { name } => commands::presets::activate(&api_url, &name).await,
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "hd", &mut io::stdout());
            Ok(())
        }
    }
}
