use anyhow::{bail, Result};

use crate::config;
use crate::format;

pub fn set(name: &str, command: &str) -> Result<()> {
    // Prevent aliasing built-in commands
    let builtins = [
        "auth",
        "status",
        "whoami",
        "busy",
        "online",
        "offline",
        "limited",
        "verdict",
        "presets",
        "preset",
        "watch",
        "doctor",
        "update",
        "hook",
        "telemetry",
        "alias",
        "completions",
        "help",
    ];

    if builtins.contains(&name) {
        bail!("Cannot alias '{}' because it's a built-in command", name);
    }

    config::update(|cfg| {
        cfg.aliases.insert(name.to_string(), command.to_string());
    })?;

    println!();
    println!(
        "  {} Alias set: {} = {}",
        format::styled_green_bold("✓"),
        format::styled_bold(name),
        format::styled_dimmed(command)
    );
    println!(
        "  {} Now you can run: hd {}",
        format::styled_dimmed("Tip:"),
        name
    );
    println!();
    Ok(())
}

pub fn remove(name: &str) -> Result<()> {
    let cfg = config::load()?;
    if !cfg.aliases.contains_key(name) {
        bail!("Alias '{}' not found", name);
    }

    config::update(|cfg| {
        cfg.aliases.remove(name);
    })?;

    println!();
    println!(
        "  {} Alias '{}' removed",
        format::styled_green_bold("✓"),
        name
    );
    println!();
    Ok(())
}

pub fn list(json: bool) -> Result<()> {
    let cfg = config::load()?;

    if json {
        println!("{}", serde_json::to_string_pretty(&cfg.aliases)?);
        return Ok(());
    }

    println!();
    if cfg.aliases.is_empty() {
        println!("  No aliases configured.");
        println!();
        println!(
            "  {} Set one with: hd alias set focus \"busy 2h\"",
            format::styled_dimmed("Tip:")
        );
    } else {
        println!("  {}", format::styled_bold("Aliases"));
        println!();
        for (name, command) in &cfg.aliases {
            println!(
                "  {} {} = {}",
                format::styled_dimmed("•"),
                format::styled_bold(name),
                command
            );
        }
    }
    println!();
    Ok(())
}
