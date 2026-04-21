mod auth;
mod client;
mod commands;
mod config;
mod contract;
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

    /// Show your availability resolution
    Availability {
        /// Optional RFC3339 timestamp to resolve at (defaults to now)
        #[arg(long)]
        at: Option<String>,
    },

    /// Manage reachability windows
    Windows {
        #[command(subcommand)]
        action: Option<WindowAction>,
    },

    /// Manage preset configurations
    Presets {
        #[command(subcommand)]
        action: Option<PresetsAction>,
    },

    /// Manage delegation grants
    Grants {
        #[command(subcommand)]
        action: Option<GrantsAction>,
    },

    /// Manage temporary availability overrides
    Override {
        #[command(subcommand)]
        action: Option<OverrideAction>,
    },

    /// Apply a preset by name or ID
    Preset {
        /// Preset name or ID
        name: String,
    },

    /// Manage digest summaries
    Digest {
        #[command(subcommand)]
        action: Option<DigestAction>,
    },

    /// Manage auto-responder text
    Autoresponder {
        #[command(subcommand)]
        action: Option<AutoResponderAction>,
    },

    /// Manage verdict threshold settings
    VerdictSettings {
        #[command(subcommand)]
        action: Option<VerdictSettingsAction>,
    },

    /// List recent task proposals
    Proposals {
        /// Number of latest proposals to fetch
        #[arg(long)]
        latest: Option<i32>,

        /// Filter by verdict decision (approved, deferred)
        #[arg(long, value_parser = ["approved", "deferred"])]
        verdict: Option<String>,
    },

    /// Evaluate whether interrupting someone is allowed
    Interrupt {
        /// User handle to evaluate
        handle: String,
    },

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
enum WindowAction {
    /// List configured windows
    List,

    /// Create a reachability window
    Create {
        /// Window label
        #[arg(long)]
        label: String,

        /// Mode (online, busy, limited, offline)
        #[arg(long, value_parser = ["online", "busy", "limited", "offline"])]
        mode: String,

        /// Days expression (for example: Mon-Fri)
        #[arg(long)]
        days: String,

        /// Start time (HH:MM:SS)
        #[arg(long)]
        start: String,

        /// End time (HH:MM:SS)
        #[arg(long)]
        end: String,

        /// Alerts policy (off, interruptable, do_not_disturb, take_a_number, after_hours)
        #[arg(long, value_parser = ["off", "interruptable", "do_not_disturb", "take_a_number", "after_hours"])]
        alerts_policy: Option<String>,

        /// Priority (higher wins)
        #[arg(long)]
        priority: Option<i32>,

        /// Auto activate this window
        #[arg(long)]
        auto_activate: Option<bool>,

        /// Enable snooze for this window
        #[arg(long)]
        snooze: Option<bool>,

        /// Set status enabled/disabled for this window
        #[arg(long)]
        status: Option<bool>,

        /// Optional status emoji
        #[arg(long)]
        status_emoji: Option<String>,

        /// Optional status text
        #[arg(long)]
        status_text: Option<String>,
    },

    /// Update a reachability window
    Update {
        /// Window id
        id: String,

        /// Window label
        #[arg(long)]
        label: Option<String>,

        /// Mode (online, busy, limited, offline)
        #[arg(long, value_parser = ["online", "busy", "limited", "offline"])]
        mode: Option<String>,

        /// Days expression (for example: Mon-Fri)
        #[arg(long)]
        days: Option<String>,

        /// Start time (HH:MM:SS)
        #[arg(long)]
        start: Option<String>,

        /// End time (HH:MM:SS)
        #[arg(long)]
        end: Option<String>,

        /// Alerts policy (off, interruptable, do_not_disturb, take_a_number, after_hours)
        #[arg(long, value_parser = ["off", "interruptable", "do_not_disturb", "take_a_number", "after_hours"])]
        alerts_policy: Option<String>,

        /// Priority (higher wins)
        #[arg(long)]
        priority: Option<i32>,

        /// Auto activate this window
        #[arg(long)]
        auto_activate: Option<bool>,

        /// Enable snooze for this window
        #[arg(long)]
        snooze: Option<bool>,

        /// Set status enabled/disabled for this window
        #[arg(long)]
        status: Option<bool>,

        /// Optional status emoji
        #[arg(long)]
        status_emoji: Option<String>,

        /// Optional status text
        #[arg(long)]
        status_text: Option<String>,
    },

    /// Delete a reachability window
    Delete {
        /// Window id
        id: String,
    },
}

#[derive(Subcommand)]
enum PresetsAction {
    /// List configured presets
    List,
}

#[derive(Subcommand)]
enum GrantsAction {
    /// List active grants
    ListActive,

    /// List grants with optional filters
    List {
        #[arg(long)]
        active: Option<bool>,
        #[arg(long, value_parser = ["session", "workspace", "agent"])]
        scope: Option<String>,
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        workspace_ref: Option<String>,
        #[arg(long)]
        agent_id: Option<String>,
        #[arg(long)]
        source: Option<String>,
    },

    /// Create a grant
    Create {
        #[arg(long, value_parser = ["session", "workspace", "agent"])]
        scope: String,
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        workspace_ref: Option<String>,
        #[arg(long)]
        agent_id: Option<String>,
        #[arg(long, value_delimiter = ',')]
        permissions: Vec<String>,
        #[arg(long)]
        duration_minutes: Option<i32>,
        #[arg(long)]
        expires_at: Option<String>,
        #[arg(long)]
        source: Option<String>,
    },

    /// Revoke one grant by id
    Revoke { id: String },

    /// Revoke many grants with optional filters
    RevokeMany {
        #[arg(long)]
        active: Option<bool>,
        #[arg(long, value_parser = ["session", "workspace", "agent"])]
        scope: Option<String>,
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        workspace_ref: Option<String>,
        #[arg(long)]
        agent_id: Option<String>,
        #[arg(long)]
        source: Option<String>,
    },
}

#[derive(Subcommand)]
enum OverrideAction {
    /// Get active override
    Get,

    /// Set a temporary override
    Set {
        #[arg(long, value_parser = ["online", "busy", "limited", "offline"])]
        mode: String,
        #[arg(long)]
        duration_minutes: Option<i32>,
        #[arg(long)]
        expires_at: Option<String>,
        #[arg(long)]
        reason: Option<String>,
    },

    /// Clear active override (or specific id)
    Clear {
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        reason: Option<String>,
    },
}

#[derive(Subcommand)]
enum DigestAction {
    /// List recent digest summaries
    List {
        /// Number of latest summaries to fetch
        #[arg(long)]
        latest: Option<i32>,
    },

    /// Dismiss a digest entry by id
    Dismiss { id: String },
}

#[derive(Subcommand)]
enum AutoResponderAction {
    /// Show current auto-responder settings
    Get,

    /// Update auto-responder text templates
    Set {
        #[arg(long)]
        busy_text: Option<String>,
        #[arg(long)]
        limited_text: Option<String>,
        #[arg(long)]
        offline_text: Option<String>,
    },
}

#[derive(Subcommand)]
enum VerdictSettingsAction {
    /// Show current verdict settings
    Get,

    /// Update verdict settings
    Set {
        /// JSON object for thresholds
        #[arg(long)]
        thresholds: Option<String>,

        /// Default delivery mode near attention deadline (auto, wrap_up, full_depth)
        #[arg(long, value_parser = ["auto", "wrap_up", "full_depth"])]
        default_wrap_up_mode: Option<String>,

        /// Minutes before attention deadline where wrap-up behavior activates
        #[arg(long)]
        wrap_up_threshold_minutes: Option<i32>,
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
        Commands::Availability { at } => commands::availability::run(&api_url, at, json).await,
        Commands::Windows { action } => match action {
            None | Some(WindowAction::List) => commands::windows::list(&api_url, json).await,
            Some(WindowAction::Create {
                label,
                mode,
                days,
                start,
                end,
                alerts_policy,
                priority,
                auto_activate,
                snooze,
                status,
                status_emoji,
                status_text,
            }) => {
                commands::windows::create(
                    &api_url,
                    commands::windows::WindowInputArgs {
                        label: Some(label),
                        mode: Some(mode),
                        days: Some(days),
                        start: Some(start),
                        end: Some(end),
                        alerts_policy,
                        priority,
                        auto_activate,
                        snooze,
                        status,
                        status_emoji,
                        status_text,
                    },
                    json,
                )
                .await
            }
            Some(WindowAction::Update {
                id,
                label,
                mode,
                days,
                start,
                end,
                alerts_policy,
                priority,
                auto_activate,
                snooze,
                status,
                status_emoji,
                status_text,
            }) => {
                commands::windows::update(
                    &api_url,
                    &id,
                    commands::windows::WindowInputArgs {
                        label,
                        mode,
                        days,
                        start,
                        end,
                        alerts_policy,
                        priority,
                        auto_activate,
                        snooze,
                        status,
                        status_emoji,
                        status_text,
                    },
                    json,
                )
                .await
            }
            Some(WindowAction::Delete { id }) => {
                commands::windows::delete(&api_url, &id, json).await
            }
        },
        Commands::Presets { action } => match action {
            None | Some(PresetsAction::List) => commands::presets::list(&api_url, json).await,
        },
        Commands::Grants { action } => match action {
            None | Some(GrantsAction::ListActive) => {
                commands::grants::list_active(&api_url, json).await
            }
            Some(GrantsAction::List {
                active,
                scope,
                session_id,
                workspace_ref,
                agent_id,
                source,
            }) => {
                commands::grants::list(
                    &api_url,
                    commands::grants::GrantsFilterArgs {
                        active,
                        scope,
                        session_id,
                        workspace_ref,
                        agent_id,
                        source,
                    },
                    json,
                )
                .await
            }
            Some(GrantsAction::Create {
                scope,
                session_id,
                workspace_ref,
                agent_id,
                permissions,
                duration_minutes,
                expires_at,
                source,
            }) => {
                commands::grants::create(
                    &api_url,
                    commands::grants::CreateGrantArgs {
                        scope: Some(scope),
                        session_id,
                        workspace_ref,
                        agent_id,
                        permissions,
                        duration_minutes,
                        expires_at,
                        source,
                    },
                    json,
                )
                .await
            }
            Some(GrantsAction::Revoke { id }) => {
                commands::grants::revoke(&api_url, &id, json).await
            }
            Some(GrantsAction::RevokeMany {
                active,
                scope,
                session_id,
                workspace_ref,
                agent_id,
                source,
            }) => {
                commands::grants::revoke_many(
                    &api_url,
                    commands::grants::GrantsFilterArgs {
                        active,
                        scope,
                        session_id,
                        workspace_ref,
                        agent_id,
                        source,
                    },
                    json,
                )
                .await
            }
        },
        Commands::Override { action } => match action {
            None | Some(OverrideAction::Get) => commands::override_cmd::get(&api_url, json).await,
            Some(OverrideAction::Set {
                mode,
                duration_minutes,
                expires_at,
                reason,
            }) => {
                commands::override_cmd::set(
                    &api_url,
                    Some(mode),
                    duration_minutes,
                    expires_at,
                    reason,
                    json,
                )
                .await
            }
            Some(OverrideAction::Clear { id, reason }) => {
                commands::override_cmd::clear(&api_url, id, reason, json).await
            }
        },
        Commands::Preset { name } => commands::presets::activate(&api_url, &name, json).await,
        Commands::Digest { action } => match action {
            None | Some(DigestAction::List { latest: None }) => {
                commands::digest::list(&api_url, None, json).await
            }
            Some(DigestAction::List { latest }) => {
                commands::digest::list(&api_url, latest, json).await
            }
            Some(DigestAction::Dismiss { id }) => {
                commands::digest::dismiss(&api_url, &id, json).await
            }
        },
        Commands::Autoresponder { action } => match action {
            None | Some(AutoResponderAction::Get) => {
                commands::autoresponder::get(&api_url, json).await
            }
            Some(AutoResponderAction::Set {
                busy_text,
                limited_text,
                offline_text,
            }) => {
                commands::autoresponder::set(&api_url, busy_text, limited_text, offline_text, json)
                    .await
            }
        },
        Commands::VerdictSettings { action } => match action {
            None | Some(VerdictSettingsAction::Get) => {
                commands::verdict_settings::get(&api_url, json).await
            }
            Some(VerdictSettingsAction::Set {
                thresholds,
                default_wrap_up_mode,
                wrap_up_threshold_minutes,
            }) => {
                commands::verdict_settings::set(
                    &api_url,
                    thresholds.as_deref(),
                    default_wrap_up_mode.as_deref(),
                    wrap_up_threshold_minutes,
                    json,
                )
                .await
            }
        },
        Commands::Proposals { latest, verdict } => {
            commands::proposals::list(&api_url, latest, verdict, json).await
        }
        Commands::Interrupt { handle } => {
            commands::interrupt::evaluate(&api_url, &handle, json).await
        }
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
        Commands::Availability { .. } => "availability",
        Commands::Windows { .. } => "windows",
        Commands::Presets { .. } => "presets",
        Commands::Grants { .. } => "grants",
        Commands::Override { .. } => "override",
        Commands::Preset { .. } => "preset",
        Commands::Digest { .. } => "digest",
        Commands::Autoresponder { .. } => "autoresponder",
        Commands::VerdictSettings { .. } => "verdict-settings",
        Commands::Proposals { .. } => "proposals",
        Commands::Interrupt { .. } => "interrupt",
        Commands::Whoami => "whoami",
        Commands::Busy { .. } => "busy",
        Commands::Online => "online",
        Commands::Offline => "offline",
        Commands::Limited { .. } => "limited",
        Commands::Verdict { .. } => "verdict",
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
