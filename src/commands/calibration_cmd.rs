use anyhow::Result;

use crate::config;
use crate::format;

pub fn enable() -> Result<()> {
    config::update(|cfg| {
        cfg.calibration.enabled = true;
    })?;

    println!();
    println!("  {} Calibration enabled", format::styled_green_bold("✓"));
    println!(
        "  {} Your verdicts will improve as HeadsDown learns from task outcomes",
        format::styled_dimmed("ℹ")
    );
    println!(
        "  {} No code content is collected, only outcome metrics (duration, file count, success)",
        format::styled_dimmed("ℹ")
    );
    println!();
    Ok(())
}

pub fn disable() -> Result<()> {
    config::update(|cfg| {
        cfg.calibration.enabled = false;
    })?;

    println!();
    println!(
        "  {} Calibration disabled. Verdicts will use default thresholds.",
        format::styled_green_bold("✓")
    );
    println!();
    Ok(())
}

pub fn status() -> Result<()> {
    let cfg = config::load()?;

    println!();
    if cfg.calibration.enabled {
        println!(
            "  {} Calibration is {}",
            format::styled_green_bold("●"),
            format::styled_green_bold("enabled")
        );
        println!(
            "  {} Task outcomes are reported to improve verdict accuracy",
            format::styled_dimmed("ℹ")
        );
    } else {
        println!(
            "  {} Calibration is {}",
            format::styled_dimmed("●"),
            format::styled_dimmed("disabled")
        );
        println!(
            "  {} Verdicts use default thresholds (no learning from your outcomes)",
            format::styled_dimmed("ℹ")
        );
    }
    println!();
    println!(
        "  {} Toggle with: hd calibration on | hd calibration off",
        format::styled_dimmed("Tip:")
    );
    println!();
    Ok(())
}
