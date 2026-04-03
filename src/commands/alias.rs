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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn with_temp_config<F: FnOnce()>(f: F) {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        f();
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    #[serial]
    fn set_rejects_builtin_commands() {
        with_temp_config(|| {
            for builtin in &["auth", "status", "help", "alias", "busy", "watch"] {
                let result = set(builtin, "something");
                assert!(result.is_err(), "Expected error for builtin '{}'", builtin);
                assert!(
                    result.unwrap_err().to_string().contains("built-in command"),
                    "Error for '{}' should mention built-in command",
                    builtin
                );
            }
        });
    }

    #[test]
    #[serial]
    fn set_accepts_custom_name() {
        with_temp_config(|| {
            set("focus", "busy 2h").unwrap();
            let cfg = config::load().unwrap();
            assert_eq!(
                cfg.aliases.get("focus").map(|s| s.as_str()),
                Some("busy 2h")
            );
        });
    }

    #[test]
    #[serial]
    fn set_overwrites_existing_alias() {
        with_temp_config(|| {
            set("focus", "busy 2h").unwrap();
            set("focus", "busy 4h").unwrap();
            let cfg = config::load().unwrap();
            assert_eq!(cfg.aliases["focus"], "busy 4h");
        });
    }

    #[test]
    #[serial]
    fn remove_errors_on_nonexistent_alias() {
        with_temp_config(|| {
            let err = remove("nope").unwrap_err();
            assert!(err.to_string().contains("not found"));
        });
    }

    #[test]
    #[serial]
    fn remove_deletes_existing_alias() {
        with_temp_config(|| {
            set("focus", "busy 2h").unwrap();
            remove("focus").unwrap();
            let cfg = config::load().unwrap();
            assert!(!cfg.aliases.contains_key("focus"));
        });
    }
}
