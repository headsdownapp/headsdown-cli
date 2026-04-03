use anyhow::Result;
use chrono::{DateTime, Utc};

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

pub async fn run(api_url: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let data = client.execute(STATUS_QUERY, None).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    let contract = &data["activeContract"];
    let calendar = &data["calendar"];
    let profile = &data["profile"];

    let name = profile["name"].as_str().unwrap_or("Unknown");
    let mode = contract["mode"]
        .as_str()
        .unwrap_or("UNKNOWN")
        .to_uppercase();

    println!();
    println!(
        "  {} {}",
        format::styled_bold("●"),
        format::styled_bold(name)
    );
    println!();
    println!(
        "  {} {}",
        format::styled_dimmed("Mode:"),
        format::color_mode(&mode)
    );

    // Status text
    if let Some(emoji) = contract["statusEmoji"].as_str() {
        if let Some(text) = contract["statusText"].as_str() {
            println!("  {} {} {}", format::styled_dimmed("Status:"), emoji, text);
        }
    } else if let Some(text) = contract["statusText"].as_str() {
        println!("  {} {}", format::styled_dimmed("Status:"), text);
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
                    format::styled_dimmed("Time:"),
                    format::styled_bold(&formatted),
                    expires_at.format("%l:%M %p").to_string().trim()
                );
            }
        }
    }

    // Work hours info
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
    }

    if contract["lock"].as_bool() == Some(true) {
        println!(
            "  {} {}",
            format::styled_dimmed("Lock:"),
            format::styled_yellow_bold("🔒 Locked")
        );
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
