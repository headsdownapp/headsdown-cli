use anyhow::Result;
use chrono::{DateTime, Utc};
use std::io::{self, Write};
use std::time::Duration;

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const WATCH_QUERY: &str = r#"
query {
    activeContract {
        mode
        statusText
        statusEmoji
        expiresAt
        duration
        lock
    }
    calendar {
        day
        endsAt
        workHours
        offHours
    }
    profile {
        name
    }
}
"#;

pub async fn run(api_url: &str) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    println!();
    println!(
        "  {} Live status (press Ctrl+C to exit)",
        format::styled_cyan_bold("→")
    );
    println!();

    // Set up Ctrl+C handler
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    });

    let mut last_mode = String::new();
    let poll_interval = Duration::from_secs(5);

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        match client.execute(WATCH_QUERY, None).await {
            Ok(data) => {
                let contract = &data["activeContract"];
                let profile = &data["profile"];
                let calendar = &data["calendar"];

                let name = profile["name"].as_str().unwrap_or("Unknown");
                let mode = contract["mode"]
                    .as_str()
                    .unwrap_or("UNKNOWN")
                    .to_uppercase();

                // Clear previous output (move cursor up and clear lines)
                // On first render, don't clear
                if !last_mode.is_empty() {
                    // Move up 6 lines and clear them
                    for _ in 0..6 {
                        print!("\x1b[A\x1b[2K");
                    }
                }

                // Render status
                println!(
                    "  {} {}",
                    format::styled_bold("●"),
                    format::styled_bold(name)
                );
                println!(
                    "  {} {}",
                    format::styled_dimmed("Mode:"),
                    format::color_mode(&mode)
                );

                // Status text
                let emoji = contract["statusEmoji"].as_str().unwrap_or("");
                let status_text = contract["statusText"].as_str().unwrap_or("");
                if !emoji.is_empty() || !status_text.is_empty() {
                    println!(
                        "  {} {} {}",
                        format::styled_dimmed("Status:"),
                        emoji,
                        status_text
                    );
                } else {
                    println!("  {} -", format::styled_dimmed("Status:"));
                }

                // Time remaining
                if let Some(expires_str) = contract["expiresAt"].as_str() {
                    if let Ok(expires_at) = expires_str.parse::<DateTime<Utc>>() {
                        let now = Utc::now();
                        let remaining = expires_at.signed_duration_since(now);
                        if remaining.num_seconds() > 0 {
                            let mins = remaining.num_minutes();
                            let secs = remaining.num_seconds() % 60;
                            let time_str = if mins >= 60 {
                                format!("{}h {:02}m {:02}s", mins / 60, mins % 60, secs)
                            } else {
                                format!("{}m {:02}s", mins, secs)
                            };
                            println!(
                                "  {} {} (until {})",
                                format::styled_dimmed("Time:"),
                                format::styled_bold(&time_str),
                                expires_at.format("%l:%M %p").to_string().trim()
                            );
                        } else {
                            println!("  {} expired", format::styled_dimmed("Time:"));
                        }
                    } else {
                        println!("  {} -", format::styled_dimmed("Time:"));
                    }
                } else {
                    println!("  {} -", format::styled_dimmed("Time:"));
                }

                // Schedule
                if let Some(true) = calendar["offHours"].as_bool() {
                    println!(
                        "  {} {}",
                        format::styled_dimmed("Schedule:"),
                        format::styled_dimmed("Off hours")
                    );
                } else if let Some(day) = calendar["day"].as_str() {
                    println!(
                        "  {} Work hours ({})",
                        format::styled_dimmed("Schedule:"),
                        capitalize(day)
                    );
                } else {
                    println!("  {} -", format::styled_dimmed("Schedule:"));
                }

                // Updated at
                println!(
                    "  {} {}",
                    format::styled_dimmed("Updated:"),
                    format::styled_dimmed(&Utc::now().format("%H:%M:%S").to_string())
                );

                io::stdout().flush().ok();

                // Detect mode changes
                if !last_mode.is_empty() && mode != last_mode {
                    // Ring the bell on mode change
                    print!("\x07");
                    io::stdout().flush().ok();
                }
                last_mode = mode;
            }
            Err(e) => {
                eprintln!("  {} API error: {}", format::styled_yellow_bold("!"), e);
            }
        }

        // Poll with 1-second ticks for smooth countdown, full refresh every 5 ticks
        for tick in 0..5 {
            if !running.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            if tick > 0 {
                // Just update the time line (move up 3 lines, rewrite time, move back down)
                // This gives a smoother countdown feel
                tokio::time::sleep(Duration::from_secs(1)).await;
            } else {
                tokio::time::sleep(poll_interval).await;
            }
        }
    }

    println!();
    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
    }
}
