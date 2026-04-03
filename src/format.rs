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
    // Always transform the text (SCOPE_DOWN -> SCOPE DOWN)
    let display = match decision.to_uppercase().as_str() {
        "SCOPE_DOWN" => "SCOPE DOWN".to_string(),
        other => other.to_string(),
    };

    if !colors_enabled() {
        return display;
    }
    match display.as_str() {
        "APPROVED" => display.green().bold().to_string(),
        "SCOPE DOWN" => display.yellow().bold().to_string(),
        "DEFERRED" => display.red().bold().to_string(),
        _ => display.bold().to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_minutes_only() {
        assert_eq!(format_duration(30), "30 min");
        assert_eq!(format_duration(59), "59 min");
        assert_eq!(format_duration(1), "1 min");
    }

    #[test]
    fn format_duration_exact_hours() {
        assert_eq!(format_duration(60), "1 hour");
        assert_eq!(format_duration(120), "2 hours");
        assert_eq!(format_duration(180), "3 hours");
    }

    #[test]
    fn format_duration_hours_and_minutes() {
        assert_eq!(format_duration(90), "1h 30m");
        assert_eq!(format_duration(150), "2h 30m");
        assert_eq!(format_duration(75), "1h 15m");
    }

    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration(0), "0 min");
    }

    #[test]
    fn color_mode_contains_mode_text() {
        // Regardless of whether colors are enabled, the mode text must be present
        assert!(color_mode("BUSY").contains("BUSY"));
        assert!(color_mode("ONLINE").contains("ONLINE"));
        assert!(color_mode("LIMITED").contains("LIMITED"));
        assert!(color_mode("OFFLINE").contains("OFFLINE"));
        assert!(color_mode("CUSTOM").contains("CUSTOM"));
    }

    #[test]
    fn color_verdict_maps_scope_down() {
        // SCOPE_DOWN gets transformed to "SCOPE DOWN" (with space)
        let result = color_verdict("SCOPE_DOWN");
        assert!(
            result.contains("SCOPE DOWN"),
            "Expected 'SCOPE DOWN' in '{}'",
            result
        );
    }

    #[test]
    fn color_verdict_contains_decision_text() {
        assert!(color_verdict("APPROVED").contains("APPROVED"));
        assert!(color_verdict("DEFERRED").contains("DEFERRED"));
        assert!(color_verdict("UNKNOWN").contains("UNKNOWN"));
    }
}
