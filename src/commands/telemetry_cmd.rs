use anyhow::Result;

use crate::config;
use crate::format;

pub fn enable() -> Result<()> {
    config::update(|cfg| {
        cfg.telemetry.enabled = true;
    })?;

    println!();
    println!(
        "  {} Telemetry enabled. Anonymous usage data helps improve the CLI",
        format::styled_green_bold("✓")
    );
    println!(
        "  {} No personal data or commands are ever sent",
        format::styled_dimmed("ℹ")
    );
    println!();
    Ok(())
}

pub fn disable() -> Result<()> {
    config::update(|cfg| {
        cfg.telemetry.enabled = false;
    })?;

    println!();
    println!("  {} Telemetry disabled", format::styled_green_bold("✓"));
    println!();
    Ok(())
}

pub fn status() -> Result<()> {
    let cfg = config::load()?;

    println!();
    if cfg.telemetry.enabled {
        println!(
            "  {} Telemetry is {}",
            format::styled_green_bold("●"),
            format::styled_green_bold("enabled")
        );
        println!(
            "  {} Only anonymous usage counts (command names, OS, version) are collected",
            format::styled_dimmed("ℹ")
        );
    } else {
        println!(
            "  {} Telemetry is {}",
            format::styled_dimmed("●"),
            format::styled_dimmed("disabled")
        );
    }
    println!();
    println!(
        "  {} Toggle with: hd telemetry on | hd telemetry off",
        format::styled_dimmed("Tip:")
    );
    println!();
    Ok(())
}
