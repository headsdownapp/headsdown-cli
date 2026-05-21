use anyhow::{anyhow, bail, Result};
use clap::ValueEnum;
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::format;

const CLAUDE_REFEREE_COMMAND: &str = r#"---
description: Verify this run locally against a HeadsDown Referee contract and print a privacy-safe receipt
allowed-tools: Bash(headsdown-claude:*), Bash(npx:*)
argument-hint: ""
---

# HeadsDown Referee

<!-- headsdown-cli managed: claude-referee-command v1 -->

Run `headsdown-claude referee` and print only the returned receipt. If that command is unavailable, run `npx -y headsdown-claude referee`.

Do not add prompts, code, logs, file paths, repository names, branch names, terminal output, or message contents to the receipt.
"#;

const CLAUDE_MARKER: &str = "headsdown-cli managed: claude-referee-command v1";
const PI_PACKAGE: &str = "git:github.com/headsdownapp/headsdown-pi";
const CODEX_MARKER_BEGIN: &str = "# <headsdown-cli managed: codex-mcp v1>";
const CODEX_MARKER_END: &str = "# </headsdown-cli managed: codex-mcp v1>";
const CODEX_BLOCK: &str = r#"# <headsdown-cli managed: codex-mcp v1>
[mcp_servers.headsdown]
command = "npx"
args = ["-y", "headsdown-claude"]
# </headsdown-cli managed: codex-mcp v1>
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IntegrationTool {
    Claude,
    Pi,
    Codex,
}

impl IntegrationTool {
    fn label(self) -> &'static str {
        match self {
            IntegrationTool::Claude => "Claude Code",
            IntegrationTool::Pi => "Pi",
            IntegrationTool::Codex => "Codex",
        }
    }

    fn slug(self) -> &'static str {
        match self {
            IntegrationTool::Claude => "claude",
            IntegrationTool::Pi => "pi",
            IntegrationTool::Codex => "codex",
        }
    }

    fn executable(self) -> &'static str {
        match self {
            IntegrationTool::Claude => "claude",
            IntegrationTool::Pi => "pi",
            IntegrationTool::Codex => "codex",
        }
    }
}

#[derive(Debug)]
pub struct IntegrationCommandOptions {
    pub tool: Option<IntegrationTool>,
    pub all: bool,
    pub dry_run: bool,
    pub yes: bool,
    pub json: bool,
}

#[derive(Debug)]
pub struct DoctorOptions {
    pub tool: Option<IntegrationTool>,
    pub all: bool,
    pub json: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ActionStatus {
    Planned,
    Installed,
    Updated,
    AlreadyCurrent,
    Removed,
    NotInstalled,
    MissingTool,
    Skipped,
}

#[derive(Debug, Serialize)]
struct ActionReport {
    tool: IntegrationTool,
    status: ActionStatus,
    message: String,
    detected: bool,
    installed: bool,
}

#[derive(Debug, Serialize)]
struct HealthReport {
    tool: IntegrationTool,
    detected: bool,
    config_present: bool,
    installed: bool,
    current: bool,
    auth_status: &'static str,
    repair_suggestion: String,
}

#[derive(Debug)]
struct ToolState {
    detected: bool,
    config_present: bool,
    installed: bool,
    current: bool,
}

pub fn install(options: IntegrationCommandOptions) -> Result<()> {
    let tools = resolve_install_tools(options.tool, options.all)?;
    if options.all {
        if !options.json {
            print_detected_tools(&tools)?;
        }
        if options.json && !options.dry_run && !options.yes {
            return print_action_reports(
                vec![ActionReport {
                    tool: IntegrationTool::Claude,
                    status: ActionStatus::Skipped,
                    message: "Pass --yes to install detected integrations in JSON mode".to_string(),
                    detected: false,
                    installed: false,
                }],
                true,
            );
        }
        if !options.dry_run
            && !options.yes
            && !confirm("Install HeadsDown integrations for detected tools?")?
        {
            return print_action_reports(
                vec![ActionReport {
                    tool: IntegrationTool::Claude,
                    status: ActionStatus::Skipped,
                    message: "No changes made".to_string(),
                    detected: false,
                    installed: false,
                }],
                options.json,
            );
        }
    }

    let mut reports = Vec::new();
    for tool in tools {
        reports.push(install_tool(tool, options.dry_run)?);
    }
    print_action_reports(reports, options.json)
}

pub fn update(options: IntegrationCommandOptions) -> Result<()> {
    let tools = resolve_update_tools(options.tool, options.all)?;
    if options.all {
        if !options.json {
            print_detected_tools(&tools)?;
        }
        if options.json && !options.dry_run && !options.yes {
            return print_action_reports(
                vec![ActionReport {
                    tool: IntegrationTool::Claude,
                    status: ActionStatus::Skipped,
                    message: "Pass --yes to update detected integrations in JSON mode".to_string(),
                    detected: false,
                    installed: false,
                }],
                true,
            );
        }
        if !options.dry_run
            && !options.yes
            && !confirm("Update HeadsDown integrations for detected tools?")?
        {
            return print_action_reports(
                vec![ActionReport {
                    tool: IntegrationTool::Claude,
                    status: ActionStatus::Skipped,
                    message: "No changes made".to_string(),
                    detected: false,
                    installed: false,
                }],
                options.json,
            );
        }
    }

    let mut reports = Vec::new();
    for tool in tools {
        reports.push(update_tool(tool, options.dry_run)?);
    }
    print_action_reports(reports, options.json)
}

pub fn remove(options: IntegrationCommandOptions) -> Result<()> {
    let tool = options.tool.ok_or_else(|| {
        anyhow!("hd remove requires a supported tool, for example: hd remove claude")
    })?;
    let report = remove_tool(tool, options.dry_run)?;
    print_action_reports(vec![report], options.json)
}

pub fn default_doctor_checks() -> Result<Vec<(String, bool, String)>> {
    let tools = installed_tools().unwrap_or_default();
    if tools.is_empty() {
        return Ok(vec![(
            "HeadsDown integrations".to_string(),
            true,
            "No installed integrations found".to_string(),
        )]);
    }

    let mut checks = Vec::new();
    for tool in tools {
        let report = health_report(tool)?;
        checks.push((
            format!("{} integration", tool.label()),
            report.current,
            if report.current {
                "Current".to_string()
            } else {
                report.repair_suggestion
            },
        ));
    }
    Ok(checks)
}

pub fn doctor(options: DoctorOptions) -> Result<()> {
    let tools = resolve_doctor_tools(options.tool, options.all)?;
    let reports: Vec<HealthReport> = tools
        .into_iter()
        .map(health_report)
        .collect::<Result<_>>()?;

    if options.json {
        println!("{}", serde_json::to_string_pretty(&reports)?);
        return Ok(());
    }

    println!();
    println!(
        "  {} HeadsDown integration health",
        format::styled_bold("HeadsDown")
    );
    println!();

    for report in reports {
        let icon = if report.current {
            format::styled_green_bold("✓")
        } else {
            format::styled_yellow_bold("!")
        };
        println!("  {} {}", icon, report.tool.label());
        println!(
            "    {} {}",
            format::styled_dimmed("Tool detected:"),
            yes_no(report.detected)
        );
        println!(
            "    {} {}",
            format::styled_dimmed("Config present:"),
            yes_no(report.config_present)
        );
        println!(
            "    {} {}",
            format::styled_dimmed("HeadsDown integration installed:"),
            yes_no(report.installed)
        );
        println!(
            "    {} {}",
            format::styled_dimmed("Integration current:"),
            yes_no(report.current)
        );
        println!(
            "    {} {}",
            format::styled_dimmed("Auth status:"),
            report.auth_status
        );
        println!(
            "    {} {}",
            format::styled_dimmed("Suggestion:"),
            report.repair_suggestion
        );
        println!();
    }

    Ok(())
}

fn resolve_install_tools(tool: Option<IntegrationTool>, all: bool) -> Result<Vec<IntegrationTool>> {
    if all && tool.is_some() {
        bail!("Pass either a tool or --all, not both.");
    }
    if let Some(tool) = tool {
        return Ok(vec![tool]);
    }
    if all {
        return detected_tools();
    }
    bail!("hd install requires a supported tool or --all, for example: hd install claude");
}

fn resolve_update_tools(tool: Option<IntegrationTool>, all: bool) -> Result<Vec<IntegrationTool>> {
    if all && tool.is_some() {
        bail!("Pass either a tool or --all, not both.");
    }
    if let Some(tool) = tool {
        return Ok(vec![tool]);
    }
    if all {
        return detected_tools();
    }
    installed_tools()
}

fn resolve_doctor_tools(tool: Option<IntegrationTool>, all: bool) -> Result<Vec<IntegrationTool>> {
    if all && tool.is_some() {
        bail!("Pass either a tool or --all, not both.");
    }
    if let Some(tool) = tool {
        return Ok(vec![tool]);
    }
    if all {
        return Ok(all_tools());
    }
    installed_tools().or_else(|_| Ok(all_tools()))
}

fn all_tools() -> Vec<IntegrationTool> {
    vec![
        IntegrationTool::Claude,
        IntegrationTool::Pi,
        IntegrationTool::Codex,
    ]
}

fn detected_tools() -> Result<Vec<IntegrationTool>> {
    Ok(all_tools()
        .into_iter()
        .filter(|tool| {
            state_for(*tool)
                .map(|state| state.detected)
                .unwrap_or(false)
        })
        .collect())
}

fn installed_tools() -> Result<Vec<IntegrationTool>> {
    let tools: Vec<IntegrationTool> = all_tools()
        .into_iter()
        .filter(|tool| {
            state_for(*tool)
                .map(|state| state.installed)
                .unwrap_or(false)
        })
        .collect();
    if tools.is_empty() {
        bail!("No installed HeadsDown integrations found. Run `hd install <tool>` first.");
    }
    Ok(tools)
}

fn install_tool(tool: IntegrationTool, dry_run: bool) -> Result<ActionReport> {
    let state = state_for(tool)?;
    if !state.detected {
        return Ok(ActionReport {
            tool,
            status: ActionStatus::MissingTool,
            message: format!("{} was not detected; no changes made", tool.label()),
            detected: false,
            installed: state.installed,
        });
    }
    if state.current {
        return Ok(ActionReport {
            tool,
            status: ActionStatus::AlreadyCurrent,
            message: "HeadsDown integration is already current".to_string(),
            detected: true,
            installed: true,
        });
    }
    if dry_run {
        return Ok(ActionReport {
            tool,
            status: ActionStatus::Planned,
            message: format!(
                "Would install the HeadsDown integration for {}",
                tool.label()
            ),
            detected: true,
            installed: state.installed,
        });
    }
    if unmanaged_conflict(tool)? {
        return Ok(ActionReport {
            tool,
            status: ActionStatus::Skipped,
            message: "Existing user-owned integration artifact was preserved; no changes made"
                .to_string(),
            detected: true,
            installed: false,
        });
    }
    write_integration(tool)?;
    Ok(ActionReport {
        tool,
        status: ActionStatus::Installed,
        message: format!("Installed the HeadsDown integration for {}", tool.label()),
        detected: true,
        installed: true,
    })
}

fn update_tool(tool: IntegrationTool, dry_run: bool) -> Result<ActionReport> {
    let state = state_for(tool)?;
    if !state.detected {
        return Ok(ActionReport {
            tool,
            status: ActionStatus::MissingTool,
            message: format!("{} was not detected; no changes made", tool.label()),
            detected: false,
            installed: state.installed,
        });
    }
    if !state.installed {
        return Ok(ActionReport {
            tool,
            status: ActionStatus::NotInstalled,
            message: "HeadsDown integration is not installed".to_string(),
            detected: true,
            installed: false,
        });
    }
    if state.current {
        return Ok(ActionReport {
            tool,
            status: ActionStatus::AlreadyCurrent,
            message: "HeadsDown integration is already current".to_string(),
            detected: true,
            installed: true,
        });
    }
    if dry_run {
        return Ok(ActionReport {
            tool,
            status: ActionStatus::Planned,
            message: format!(
                "Would refresh the HeadsDown integration for {}",
                tool.label()
            ),
            detected: true,
            installed: state.installed,
        });
    }
    if unmanaged_conflict(tool)? {
        return Ok(ActionReport {
            tool,
            status: ActionStatus::Skipped,
            message: "Existing user-owned integration artifact was preserved; no changes made"
                .to_string(),
            detected: true,
            installed: false,
        });
    }
    write_integration(tool)?;
    Ok(ActionReport {
        tool,
        status: ActionStatus::Updated,
        message: format!("Updated the HeadsDown integration for {}", tool.label()),
        detected: true,
        installed: true,
    })
}

fn remove_tool(tool: IntegrationTool, dry_run: bool) -> Result<ActionReport> {
    let state = state_for(tool)?;
    if !state.installed {
        return Ok(ActionReport {
            tool,
            status: ActionStatus::NotInstalled,
            message: "No HeadsDown integration was installed".to_string(),
            detected: state.detected,
            installed: false,
        });
    }
    if dry_run {
        return Ok(ActionReport {
            tool,
            status: ActionStatus::Planned,
            message: format!(
                "Would remove the HeadsDown integration for {}",
                tool.label()
            ),
            detected: state.detected,
            installed: true,
        });
    }
    remove_integration(tool)?;
    Ok(ActionReport {
        tool,
        status: ActionStatus::Removed,
        message: format!("Removed the HeadsDown integration for {}", tool.label()),
        detected: state.detected,
        installed: false,
    })
}

fn health_report(tool: IntegrationTool) -> Result<HealthReport> {
    let state = state_for(tool)?;
    let auth_status = if crate::auth::load_token().ok().flatten().is_some() {
        "present"
    } else {
        "not_found"
    };
    let repair_suggestion = if state.current {
        "No repair needed".to_string()
    } else if state.installed {
        format!("Run `hd update {}`", tool.slug())
    } else if state.detected {
        format!("Run `hd install {}`", tool.slug())
    } else {
        format!(
            "Install {} first, then run `hd install {}`",
            tool.label(),
            tool.slug()
        )
    };

    Ok(HealthReport {
        tool,
        detected: state.detected,
        config_present: state.config_present,
        installed: state.installed,
        current: state.current,
        auth_status,
        repair_suggestion,
    })
}

fn state_for(tool: IntegrationTool) -> Result<ToolState> {
    match tool {
        IntegrationTool::Claude => claude_state(),
        IntegrationTool::Pi => pi_state(),
        IntegrationTool::Codex => codex_state(),
    }
}

fn unmanaged_conflict(tool: IntegrationTool) -> Result<bool> {
    match tool {
        IntegrationTool::Claude => {
            let path = claude_command_path()?;
            Ok(path.exists()
                && fs::read_to_string(path)
                    .map(|content| !content.contains(CLAUDE_MARKER))
                    .unwrap_or(true))
        }
        IntegrationTool::Pi => Ok(false),
        IntegrationTool::Codex => {
            let path = codex_config_path()?;
            let content = fs::read_to_string(path).unwrap_or_default();
            let unmanaged_content = content_without_managed_codex_block(&content)?;
            Ok(unmanaged_content.contains("[mcp_servers.headsdown]"))
        }
    }
}

fn write_integration(tool: IntegrationTool) -> Result<()> {
    match tool {
        IntegrationTool::Claude => write_claude(),
        IntegrationTool::Pi => write_pi(),
        IntegrationTool::Codex => write_codex(),
    }
}

fn remove_integration(tool: IntegrationTool) -> Result<()> {
    match tool {
        IntegrationTool::Claude => remove_claude(),
        IntegrationTool::Pi => remove_pi(),
        IntegrationTool::Codex => remove_codex(),
    }
}

fn claude_state() -> Result<ToolState> {
    let command_path = claude_command_path()?;
    let config_dir = claude_config_dir()?;
    let content = fs::read_to_string(&command_path).ok();
    let installed = content
        .as_ref()
        .map(|value| value.contains(CLAUDE_MARKER))
        .unwrap_or(false);
    let current = content
        .as_ref()
        .map(|value| value == CLAUDE_REFEREE_COMMAND)
        .unwrap_or(false);
    Ok(ToolState {
        detected: config_dir.exists() || executable_exists(IntegrationTool::Claude.executable()),
        config_present: config_dir.exists(),
        installed,
        current,
    })
}

fn write_claude() -> Result<()> {
    let path = claude_command_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, CLAUDE_REFEREE_COMMAND)?;
    Ok(())
}

fn remove_claude() -> Result<()> {
    let path = claude_command_path()?;
    if let Ok(content) = fs::read_to_string(&path) {
        if content.contains(CLAUDE_MARKER) {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn pi_state() -> Result<ToolState> {
    let dir = pi_config_dir()?;
    let path = dir.join("settings.json");
    let content = fs::read_to_string(&path).ok();
    let installed = content
        .as_deref()
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
        .and_then(|value| {
            value
                .get("packages")
                .and_then(|packages| packages.as_array())
                .cloned()
        })
        .map(|packages| {
            packages
                .iter()
                .any(|package| package.as_str() == Some(PI_PACKAGE))
        })
        .unwrap_or(false);
    Ok(ToolState {
        detected: dir.exists()
            || path.exists()
            || executable_exists(IntegrationTool::Pi.executable()),
        config_present: dir.exists() || path.exists(),
        installed,
        current: installed,
    })
}

fn write_pi() -> Result<()> {
    let path = pi_settings_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut value = if path.exists() {
        serde_json::from_str::<Value>(&fs::read_to_string(&path)?)?
    } else {
        serde_json::json!({})
    };
    let object = value
        .as_object_mut()
        .ok_or_else(|| anyhow!("Pi settings must be a JSON object."))?;
    let packages_value = object
        .entry("packages".to_string())
        .or_insert_with(|| serde_json::json!([]));
    let packages = packages_value
        .as_array_mut()
        .ok_or_else(|| anyhow!("Pi settings packages must be a JSON array."))?;
    if !packages
        .iter()
        .any(|package| package.as_str() == Some(PI_PACKAGE))
    {
        packages.push(Value::String(PI_PACKAGE.to_string()));
    }
    fs::write(path, serde_json::to_string_pretty(&value)? + "\n")?;
    Ok(())
}

fn remove_pi() -> Result<()> {
    let path = pi_settings_path()?;
    if !path.exists() {
        return Ok(());
    }
    let mut value = serde_json::from_str::<Value>(&fs::read_to_string(&path)?)?;
    if let Some(packages) = value
        .get_mut("packages")
        .and_then(|packages| packages.as_array_mut())
    {
        packages.retain(|package| package.as_str() != Some(PI_PACKAGE));
        fs::write(path, serde_json::to_string_pretty(&value)? + "\n")?;
    }
    Ok(())
}

fn codex_state() -> Result<ToolState> {
    let dir = codex_config_dir()?;
    let path = dir.join("config.toml");
    let content = fs::read_to_string(&path).ok();
    let installed = content
        .as_ref()
        .map(|value| value.contains(CODEX_MARKER_BEGIN) && value.contains(CODEX_MARKER_END))
        .unwrap_or(false);
    let current = content
        .as_ref()
        .map(|value| {
            value.contains(CODEX_BLOCK)
                && content_without_managed_codex_block(value)
                    .map(|stripped| !stripped.contains("[mcp_servers.headsdown]"))
                    .unwrap_or(false)
        })
        .unwrap_or(false);
    Ok(ToolState {
        detected: dir.exists()
            || path.exists()
            || executable_exists(IntegrationTool::Codex.executable()),
        config_present: dir.exists() || path.exists(),
        installed,
        current,
    })
}

fn write_codex() -> Result<()> {
    let path = codex_config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let stripped = remove_managed_block(&existing)?;
    let mut next = stripped.trim_end().to_string();
    if !next.is_empty() {
        next.push_str("\n\n");
    }
    next.push_str(CODEX_BLOCK);
    fs::write(path, next)?;
    Ok(())
}

fn remove_codex() -> Result<()> {
    let path = codex_config_path()?;
    if !path.exists() {
        return Ok(());
    }
    let existing = fs::read_to_string(&path)?;
    let next = remove_managed_block(&existing)?;
    fs::write(path, next.trim_start())?;
    Ok(())
}

fn content_without_managed_codex_block(content: &str) -> Result<String> {
    if content.contains(CODEX_MARKER_BEGIN) || content.contains(CODEX_MARKER_END) {
        remove_managed_block(content)
    } else {
        Ok(content.to_string())
    }
}

fn remove_managed_block(content: &str) -> Result<String> {
    let begin = content.find(CODEX_MARKER_BEGIN);
    let Some(begin) = begin else {
        if content.contains(CODEX_MARKER_END) {
            bail!("Found an incomplete HeadsDown-managed Codex block; no changes made.");
        }
        return Ok(content.to_string());
    };

    let search_start = begin + CODEX_MARKER_BEGIN.len();
    let Some(relative_end) = content[search_start..].find(CODEX_MARKER_END) else {
        bail!("Found an incomplete HeadsDown-managed Codex block; no changes made.");
    };
    let end = search_start + relative_end + CODEX_MARKER_END.len();

    let mut output = String::new();
    output.push_str(content[..begin].trim_end());
    if !output.is_empty() {
        output.push('\n');
    }
    let suffix = content[end..].trim_start();
    if !suffix.is_empty() {
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(suffix);
    }
    Ok(output)
}

fn print_action_reports(reports: Vec<ActionReport>, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(&reports)?);
        return Ok(());
    }
    println!();
    for report in reports {
        let icon = match report.status {
            ActionStatus::Installed
            | ActionStatus::Updated
            | ActionStatus::AlreadyCurrent
            | ActionStatus::Removed => format::styled_green_bold("✓"),
            ActionStatus::Planned | ActionStatus::Skipped => format::styled_cyan_bold("→"),
            ActionStatus::MissingTool | ActionStatus::NotInstalled => {
                format::styled_yellow_bold("!")
            }
        };
        println!("  {} {}", icon, report.message);
    }
    println!();
    Ok(())
}

fn print_detected_tools(tools: &[IntegrationTool]) -> Result<()> {
    println!();
    if tools.is_empty() {
        println!(
            "  {} No supported local tools were detected",
            format::styled_yellow_bold("!")
        );
    } else {
        println!(
            "  {} Detected supported tools:",
            format::styled_cyan_bold("→")
        );
        for tool in tools {
            println!("    - {}", tool.label());
        }
    }
    println!();
    Ok(())
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("  {} {} [y/N] ", format::styled_cyan_bold("?"), prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(matches!(
        input.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn executable_exists(name: &str) -> bool {
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path_var).any(|dir| {
        let candidate = dir.join(name);
        candidate.is_file() || candidate.with_extension("exe").is_file()
    })
}

fn claude_config_dir() -> Result<PathBuf> {
    env_or_home("CLAUDE_CONFIG_HOME", ".claude")
}

fn claude_command_path() -> Result<PathBuf> {
    Ok(claude_config_dir()?
        .join("commands")
        .join("headsdown")
        .join("referee.md"))
}

fn pi_config_dir() -> Result<PathBuf> {
    env_or_home("PI_AGENT_CONFIG_HOME", ".pi/agent")
}

fn pi_settings_path() -> Result<PathBuf> {
    Ok(pi_config_dir()?.join("settings.json"))
}

fn codex_config_dir() -> Result<PathBuf> {
    env_or_home("CODEX_HOME", ".codex")
}

fn codex_config_path() -> Result<PathBuf> {
    Ok(codex_config_dir()?.join("config.toml"))
}

fn env_or_home(env_name: &str, relative: &str) -> Result<PathBuf> {
    if let Some(value) = std::env::var_os(env_name) {
        return Ok(PathBuf::from(value));
    }
    let home =
        std::env::var_os("HOME").ok_or_else(|| anyhow!("Could not determine home directory."))?;
    Ok(PathBuf::from(home).join(relative))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    fn with_home<T>(f: impl FnOnce(&TempDir) -> T) -> T {
        let dir = tempfile::tempdir().unwrap();
        let old_home = std::env::var_os("HOME");
        let old_claude = std::env::var_os("CLAUDE_CONFIG_HOME");
        let old_pi = std::env::var_os("PI_AGENT_CONFIG_HOME");
        let old_codex = std::env::var_os("CODEX_HOME");
        std::env::set_var("HOME", dir.path());
        std::env::remove_var("CLAUDE_CONFIG_HOME");
        std::env::remove_var("PI_AGENT_CONFIG_HOME");
        std::env::remove_var("CODEX_HOME");
        let result = f(&dir);
        restore_env("HOME", old_home);
        restore_env("CLAUDE_CONFIG_HOME", old_claude);
        restore_env("PI_AGENT_CONFIG_HOME", old_pi);
        restore_env("CODEX_HOME", old_codex);
        result
    }

    fn restore_env(name: &str, value: Option<std::ffi::OsString>) {
        if let Some(value) = value {
            std::env::set_var(name, value);
        } else {
            std::env::remove_var(name);
        }
    }

    #[test]
    #[serial]
    fn claude_install_is_idempotent_and_removable() {
        with_home(|dir| {
            fs::create_dir_all(dir.path().join(".claude")).unwrap();
            let first = install_tool(IntegrationTool::Claude, false).unwrap();
            let second = install_tool(IntegrationTool::Claude, false).unwrap();
            assert_eq!(first.status, ActionStatus::Installed);
            assert_eq!(second.status, ActionStatus::AlreadyCurrent);
            assert!(claude_command_path().unwrap().exists());
            let removed = remove_tool(IntegrationTool::Claude, false).unwrap();
            assert_eq!(removed.status, ActionStatus::Removed);
            assert!(!claude_command_path().unwrap().exists());
        });
    }

    #[test]
    #[serial]
    fn pi_install_preserves_existing_packages() {
        with_home(|dir| {
            let agent_dir = dir.path().join(".pi/agent");
            fs::create_dir_all(&agent_dir).unwrap();
            fs::write(
                agent_dir.join("settings.json"),
                r#"{"packages":["existing"]}"#,
            )
            .unwrap();
            install_tool(IntegrationTool::Pi, false).unwrap();
            let raw = fs::read_to_string(agent_dir.join("settings.json")).unwrap();
            let value: Value = serde_json::from_str(&raw).unwrap();
            let packages = value["packages"].as_array().unwrap();
            assert!(packages
                .iter()
                .any(|package| package.as_str() == Some("existing")));
            assert!(packages
                .iter()
                .any(|package| package.as_str() == Some(PI_PACKAGE)));
            remove_tool(IntegrationTool::Pi, false).unwrap();
            let raw = fs::read_to_string(agent_dir.join("settings.json")).unwrap();
            assert!(!raw.contains(PI_PACKAGE));
            assert!(raw.contains("existing"));
        });
    }

    #[test]
    fn codex_remove_only_managed_block() {
        let raw = "keep = true\n\n# <headsdown-cli managed: codex-mcp v1>\nremove = true\n# </headsdown-cli managed: codex-mcp v1>\n\nkeep_again = true\n";
        let stripped = remove_managed_block(raw).unwrap();
        assert!(stripped.contains("keep = true"));
        assert!(stripped.contains("keep_again = true"));
        assert!(!stripped.contains("remove = true"));
    }

    #[test]
    fn codex_incomplete_marker_is_not_stripped() {
        let raw = "keep = true\n# <headsdown-cli managed: codex-mcp v1>\nuser_owned = true\n";
        assert!(remove_managed_block(raw).is_err());
    }
}
