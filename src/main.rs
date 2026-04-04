mod auth;
mod client;
mod commands;
mod config;
mod format;
mod telemetry;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io;

/// HeadsDown CLI — manage your availability from the terminal
#[derive(Parser)]
#[command(name = "hd", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// API base URL (defaults to https://headsdown.app)
    #[arg(long, global = true, env = "HEADSDOWN_API_URL")]
    api_url: Option<String>,

    /// Output as JSON (for scripting)
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with HeadsDown via Device Flow
    Auth,

    /// Show your current availability status
    Status,

    /// Show your authenticated identity
    Whoami,

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

    /// Live-updating status dashboard
    Watch,

    /// Check CLI health and connectivity
    Doctor,

    /// Update the CLI to the latest version
    Update,

    /// Manage git hook integration
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },

    /// Manage anonymous usage telemetry
    Telemetry {
        #[command(subcommand)]
        action: TelemetryAction,
    },

    /// Manage calibration reporting (improves verdict accuracy)
    Calibration {
        #[command(subcommand)]
        action: CalibrationAction,
    },

    /// Report the outcome of an agent task
    Outcome {
        /// Proposal ID from the verdict
        proposal_id: String,

        /// What happened (completed, failed, partially_completed, cancelled, timed_out)
        #[arg(value_parser = ["completed", "failed", "partially_completed", "cancelled", "timed_out"])]
        outcome: String,

        /// Duration in minutes
        #[arg(long, short = 'd')]
        duration: Option<i32>,

        /// Files modified
        #[arg(long, short = 'f')]
        files: Option<i32>,

        /// Lines changed
        #[arg(long, short = 'l')]
        lines: Option<i32>,

        /// Turn count
        #[arg(long, short = 't')]
        turns: Option<i32>,

        /// Error category if failed
        #[arg(long)]
        error_category: Option<String>,

        /// Whether tests passed
        #[arg(long)]
        tests_passed: Option<bool>,
    },

    /// Manage command aliases
    Alias {
        #[command(subcommand)]
        action: AliasAction,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },

    /// Generate man pages to a directory
    #[command(hide = true)]
    Manpages {
        /// Output directory
        dir: String,
    },
}

#[derive(Subcommand)]
enum HookAction {
    /// Install git hooks in the current repository
    Install,
    /// Remove git hooks from the current repository
    Uninstall,
    /// Show git hook status for the current repository
    Status,
}

#[derive(Subcommand)]
enum TelemetryAction {
    /// Enable anonymous usage telemetry
    On,
    /// Disable anonymous usage telemetry
    Off,
    /// Show current telemetry status
    Status,
}

#[derive(Subcommand)]
enum CalibrationAction {
    /// Enable calibration reporting
    On,
    /// Disable calibration reporting
    Off,
    /// Show current calibration status
    Status,
}

#[derive(Subcommand)]
enum AliasAction {
    /// Set an alias (e.g. hd alias set focus "busy 2h")
    Set {
        /// Alias name
        name: String,
        /// Command to alias (e.g. "busy 2h")
        command: String,
    },
    /// Remove an alias
    Remove {
        /// Alias name
        name: String,
    },
    /// List all aliases
    List,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Check for alias expansion before parsing
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        if let Ok(cfg) = config::load() {
            if let Some(expansion) = cfg.aliases.get(&args[1]) {
                // Rebuild args: [binary, ...expanded_words, ...remaining_args]
                let mut new_args = vec![args[0].clone()];
                new_args.extend(expansion.split_whitespace().map(String::from));
                new_args.extend_from_slice(&args[2..]);
                // Re-parse with expanded args
                let cli = Cli::parse_from(&new_args);
                return dispatch(cli).await;
            }
        }
    }

    let cli = Cli::parse();
    dispatch(cli).await
}

async fn dispatch(cli: Cli) -> anyhow::Result<()> {
    let json = cli.json;

    // Resolve API URL: flag > config > default
    let api_url = match cli.api_url {
        Some(url) => url,
        None => {
            let cfg = config::load().unwrap_or_default();
            cfg.api_url
                .unwrap_or_else(|| "https://headsdown.app".to_string())
        }
    };

    // Track command usage (async, non-blocking)
    if let Commands::Completions { .. } | Commands::Manpages { .. } = &cli.command {
        // Skip telemetry for meta commands
    } else {
        let cmd_name = command_name(&cli.command);
        telemetry::track(cmd_name).await;
    }

    match cli.command {
        Commands::Auth => commands::auth::run(&api_url).await,
        Commands::Status => commands::status::run(&api_url, json).await,
        Commands::Whoami => commands::whoami::run(&api_url, json).await,
        Commands::Busy { duration } => commands::mode::run(&api_url, "BUSY", duration, json).await,
        Commands::Online => commands::mode::run(&api_url, "ONLINE", None, json).await,
        Commands::Offline => commands::mode::run(&api_url, "OFFLINE", None, json).await,
        Commands::Limited { duration } => {
            commands::mode::run(&api_url, "LIMITED", duration, json).await
        }
        Commands::Verdict {
            description,
            files,
            minutes,
            model,
        } => commands::verdict::run(&api_url, &description, files, minutes, model, json).await,
        Commands::Presets => commands::presets::run(&api_url, json).await,
        Commands::Preset { name } => commands::presets::activate(&api_url, &name, json).await,
        Commands::Watch => commands::watch::run(&api_url).await,
        Commands::Doctor => commands::doctor::run(&api_url, json).await,
        Commands::Update => commands::update::run().await,
        Commands::Hook { action } => match action {
            HookAction::Install => commands::hooks::install(),
            HookAction::Uninstall => commands::hooks::uninstall(),
            HookAction::Status => commands::hooks::status(),
        },
        Commands::Telemetry { action } => match action {
            TelemetryAction::On => commands::telemetry_cmd::enable(),
            TelemetryAction::Off => commands::telemetry_cmd::disable(),
            TelemetryAction::Status => commands::telemetry_cmd::status(),
        },
        Commands::Calibration { action } => match action {
            CalibrationAction::On => commands::calibration_cmd::enable(),
            CalibrationAction::Off => commands::calibration_cmd::disable(),
            CalibrationAction::Status => commands::calibration_cmd::status(),
        },
        Commands::Outcome {
            proposal_id,
            outcome,
            duration,
            files,
            lines,
            turns,
            error_category,
            tests_passed,
        } => {
            commands::outcome::run(
                &api_url,
                &proposal_id,
                &outcome,
                duration,
                files,
                lines,
                turns,
                error_category,
                tests_passed,
                json,
            )
            .await
        }
        Commands::Alias { action } => match action {
            AliasAction::Set { name, command } => commands::alias::set(&name, &command),
            AliasAction::Remove { name } => commands::alias::remove(&name),
            AliasAction::List => commands::alias::list(json),
        },
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "hd", &mut io::stdout());
            Ok(())
        }
        Commands::Manpages { dir } => {
            let cmd = Cli::command();
            let man = clap_mangen::Man::new(cmd);
            let mut buffer: Vec<u8> = Vec::new();
            man.render(&mut buffer)?;
            std::fs::create_dir_all(&dir)?;
            std::fs::write(format!("{}/hd.1", dir), buffer)?;
            println!("Man page written to {}/hd.1", dir);
            Ok(())
        }
    }
}

fn command_name(cmd: &Commands) -> &'static str {
    match cmd {
        Commands::Auth => "auth",
        Commands::Status => "status",
        Commands::Whoami => "whoami",
        Commands::Busy { .. } => "busy",
        Commands::Online => "online",
        Commands::Offline => "offline",
        Commands::Limited { .. } => "limited",
        Commands::Verdict { .. } => "verdict",
        Commands::Presets => "presets",
        Commands::Preset { .. } => "preset",
        Commands::Watch => "watch",
        Commands::Doctor => "doctor",
        Commands::Update => "update",
        Commands::Hook { .. } => "hook",
        Commands::Telemetry { .. } => "telemetry",
        Commands::Calibration { .. } => "calibration",
        Commands::Outcome { .. } => "outcome",
        Commands::Alias { .. } => "alias",
        Commands::Completions { .. } => "completions",
        Commands::Manpages { .. } => "manpages",
    }
}
