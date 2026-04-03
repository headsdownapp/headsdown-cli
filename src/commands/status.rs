use anyhow::Result;
use chrono::{DateTime, Utc};
use owo_colors::OwoColorize;

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const STATUS_QUERY: &str = r#"
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

    let data = client.execute(STATUS_QUERY, None).await?;

    let contract = &data["activeContract"];
    let calendar = &data["calendar"];
    let profile = &data["profile"];

    let name = profile["name"].as_str().unwrap_or("Unknown");
    let mode = contract["mode"]
        .as_str()
        .unwrap_or("UNKNOWN")
        .to_uppercase();

    println!();
    println!("  {} {}", "●".bold(), name.bold());
    println!();
    println!("  {} {}", "Mode:".dimmed(), format::color_mode(&mode));

    // Status text
    if let Some(emoji) = contract["statusEmoji"].as_str() {
        if let Some(text) = contract["statusText"].as_str() {
            println!("  {} {} {}", "Status:".dimmed(), emoji, text);
        }
    } else if let Some(text) = contract["statusText"].as_str() {
        println!("  {} {}", "Status:".dimmed(), text);
    }

    // Duration / expires at
    if let Some(expires_str) = contract["expiresAt"].as_str() {
        if let Ok(expires_at) = expires_str.parse::<DateTime<Utc>>() {
            let now = Utc::now();
            let remaining = expires_at.signed_duration_since(now);

            if remaining.num_minutes() > 0 {
                let formatted = format::format_duration(remaining.num_minutes());
                println!(
                    "  {} {} remaining (until {})",
                    "Time:".dimmed(),
                    formatted.bold(),
                    expires_at.format("%l:%M %p").to_string().trim()
                );
            }
        }
    }

    // Work hours info
    if let Some(true) = calendar["offHours"].as_bool() {
        println!("  {} {}", "Schedule:".dimmed(), "Off hours".dimmed());
    } else if let Some(day) = calendar["day"].as_str() {
        println!(
            "  {} Work hours ({})",
            "Schedule:".dimmed(),
            capitalize(day)
        );
    }

    if contract["lock"].as_bool() == Some(true) {
        println!("  {} {}", "Lock:".dimmed(), "🔒 Locked".yellow());
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
