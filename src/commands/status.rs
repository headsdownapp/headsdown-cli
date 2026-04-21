use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auth;
use crate::client::GraphQLClient;
use crate::contract::availability::AvailabilityResolution;
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
    availability {
        inReachableHours
        nextTransitionAt
        activeWindow {
            label
            mode
        }
        nextWindow {
            label
            mode
        }
    }
    profile {
        name
    }
}
"#;

#[derive(Deserialize, Serialize)]
struct StatusResponse {
    #[serde(rename = "activeContract")]
    active_contract: Option<ActiveContract>,
    availability: Option<AvailabilityResolution>,
    profile: Option<Profile>,
}

#[derive(Deserialize, Serialize)]
struct ActiveContract {
    mode: Option<String>,
    #[serde(rename = "statusText")]
    status_text: Option<String>,
    #[serde(rename = "statusEmoji")]
    status_emoji: Option<String>,
    #[serde(rename = "expiresAt")]
    expires_at: Option<String>,
    lock: Option<bool>,
}

#[derive(Deserialize, Serialize)]
struct Profile {
    name: Option<String>,
}

pub async fn run(api_url: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let data: StatusResponse = client.execute_typed(STATUS_QUERY, None).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    let contract = data.active_contract;
    let availability = data.availability;
    let profile = data.profile;

    let name = profile
        .and_then(|p| p.name)
        .unwrap_or_else(|| "Unknown".to_string());
    let mode = contract
        .as_ref()
        .and_then(|c| c.mode.clone())
        .unwrap_or_else(|| "UNKNOWN".to_string())
        .to_uppercase();

    println!();
    println!(
        "  {} {}",
        format::styled_bold("●"),
        format::styled_bold(&name)
    );
    println!();
    println!(
        "  {} {}",
        format::styled_dimmed("Mode:"),
        format::color_mode(&mode)
    );

    if let Some(text) = contract.as_ref().and_then(|c| c.status_text.clone()) {
        if let Some(emoji) = contract.as_ref().and_then(|c| c.status_emoji.clone()) {
            println!("  {} {} {}", format::styled_dimmed("Status:"), emoji, text);
        } else {
            println!("  {} {}", format::styled_dimmed("Status:"), text);
        }
    }

    if let Some(expires_str) = contract.as_ref().and_then(|c| c.expires_at.clone()) {
        if let Ok(expires_at) = expires_str.parse::<DateTime<Utc>>() {
            let remaining = expires_at.signed_duration_since(Utc::now());
            if remaining.num_minutes() > 0 {
                println!(
                    "  {} {} remaining (until {})",
                    format::styled_dimmed("Time:"),
                    format::styled_bold(&format::format_duration(remaining.num_minutes())),
                    expires_at.format("%l:%M %p").to_string().trim()
                );
            }
        }
    }

    if let Some(in_hours) = availability.as_ref().and_then(|a| a.in_reachable_hours) {
        println!(
            "  {} {}",
            format::styled_dimmed("Availability:"),
            if in_hours {
                "Reachable now"
            } else {
                "Not reachable now"
            }
        );
    }

    if let Some(label) = availability
        .as_ref()
        .and_then(|a| a.active_window.as_ref())
        .and_then(|w| w.label.clone())
    {
        println!("  {} {}", format::styled_dimmed("Window:"), label);
    }

    if let Some(next_transition) = availability
        .as_ref()
        .and_then(|a| a.next_transition_at.clone())
    {
        if let Ok(next_at) = next_transition.parse::<DateTime<Utc>>() {
            println!(
                "  {} {}",
                format::styled_dimmed("Next change:"),
                next_at.format("%l:%M %p").to_string().trim()
            );
        }
    }

    if contract.as_ref().and_then(|c| c.lock) == Some(true) {
        println!(
            "  {} {}",
            format::styled_dimmed("Lock:"),
            format::styled_yellow_bold("🔒 Locked")
        );
    }

    println!();
    Ok(())
}
