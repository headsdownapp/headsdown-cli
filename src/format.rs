use owo_colors::OwoColorize;
use std::sync::OnceLock;

/// Whether stdout supports color output.
/// Checked once, then cached. Respects NO_COLOR env var and TTY detection.
fn colors_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        // Respect the NO_COLOR convention (https://no-color.org)
        if std::env::var_os("NO_COLOR").is_some() {
            return false;
        }
        // Respect FORCE_COLOR for CI environments that support color
        if std::env::var_os("FORCE_COLOR").is_some() {
            return true;
        }
        // Check if stdout is a terminal
        atty::is(atty::Stream::Stdout)
    })
}

/// Color a mode string based on its type.
/// Falls back to plain text when stdout is not a TTY.
pub fn color_mode(mode: &str) -> String {
    if !colors_enabled() {
        return mode.to_string();
    }
    match mode.to_uppercase().as_str() {
        "ONLINE" => mode.green().bold().to_string(),
        "BUSY" => mode.red().bold().to_string(),
        "LIMITED" => mode.yellow().bold().to_string(),
        "OFFLINE" => mode.dimmed().bold().to_string(),
        _ => mode.bold().to_string(),
    }
}

/// Color a verdict decision string.
pub fn color_verdict(decision: &str) -> String {
    if !colors_enabled() {
        return decision.to_string();
    }
    match decision.to_uppercase().as_str() {
        "APPROVED" => decision.green().bold().to_string(),
        "SCOPE_DOWN" => "SCOPE DOWN".yellow().bold().to_string(),
        "DEFERRED" => decision.red().bold().to_string(),
        _ => decision.bold().to_string(),
    }
}

/// Format a duration in minutes to a human-readable string.
pub fn format_duration(minutes: i64) -> String {
    if minutes < 60 {
        format!("{} min", minutes)
    } else {
        let hours = minutes / 60;
        let remaining = minutes % 60;
        if remaining == 0 {
            if hours == 1 {
                "1 hour".to_string()
            } else {
                format!("{} hours", hours)
            }
        } else {
            format!("{}h {}m", hours, remaining)
        }
    }
}

// Helpers for consistent styled output across commands.
// These respect TTY detection and NO_COLOR automatically.

pub fn styled_green_bold(text: &str) -> String {
    if colors_enabled() {
        text.green().bold().to_string()
    } else {
        text.to_string()
    }
}

pub fn styled_yellow_bold(text: &str) -> String {
    if colors_enabled() {
        text.yellow().bold().to_string()
    } else {
        text.to_string()
    }
}

pub fn styled_cyan_bold(text: &str) -> String {
    if colors_enabled() {
        text.cyan().bold().to_string()
    } else {
        text.to_string()
    }
}

pub fn styled_cyan_underline(text: &str) -> String {
    if colors_enabled() {
        text.cyan().underline().to_string()
    } else {
        text.to_string()
    }
}

pub fn styled_bold(text: &str) -> String {
    if colors_enabled() {
        text.bold().to_string()
    } else {
        text.to_string()
    }
}

pub fn styled_dimmed(text: &str) -> String {
    if colors_enabled() {
        text.dimmed().to_string()
    } else {
        text.to_string()
    }
}
