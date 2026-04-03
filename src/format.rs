use owo_colors::OwoColorize;

/// Color a mode string based on its type.
pub fn color_mode(mode: &str) -> String {
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

/// Print a labeled line with dimmed label and bright value.
#[allow(dead_code)]
pub fn print_field(label: &str, value: &str) {
    println!("  {} {}", format!("{}:", label).dimmed(), value);
}
