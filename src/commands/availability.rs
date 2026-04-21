use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auth;
use crate::client::GraphQLClient;
use crate::contract::availability::{format_days, AvailabilityResolution};
use crate::format;

const AVAILABILITY_QUERY: &str = r#"
query Availability($at: DateTime) {
    availability(at: $at) {
        inReachableHours
        nextTransitionAt
        activeWindow {
            id
            label
            mode
            startTime
            endTime
            days
        }
        nextWindow {
            id
            label
            mode
            startTime
            endTime
            days
        }
    }
}
"#;

#[derive(Deserialize, Serialize)]
struct AvailabilityResponse {
    availability: Option<AvailabilityResolution>,
}

pub async fn run(api_url: &str, at: Option<String>, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let variables = serde_json::json!({ "at": at });
    let data: AvailabilityResponse = client
        .execute_typed(AVAILABILITY_QUERY, Some(variables))
        .await?;
    let availability = data
        .availability
        .ok_or_else(|| anyhow!("No availability found"))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&availability)?);
        return Ok(());
    }

    println!();
    println!("  {}", format::styled_bold("Availability"));
    println!();

    if let Some(in_hours) = availability.in_reachable_hours {
        println!(
            "  {} {}",
            format::styled_dimmed("State:"),
            if in_hours {
                "Reachable now"
            } else {
                "Not reachable now"
            }
        );
    }

    if let Some(active_window) = availability.active_window {
        if let Some(label) = active_window.label {
            let mode = active_window
                .mode
                .unwrap_or_else(|| "UNKNOWN".to_string())
                .to_uppercase();
            println!(
                "  {} {} ({})",
                format::styled_dimmed("Active:"),
                label,
                format::color_mode(&mode)
            );
            println!(
                "  {} {} {}-{}",
                format::styled_dimmed("Hours:"),
                format_days(active_window.days.as_ref()),
                active_window.start_time.unwrap_or_else(|| "-".to_string()),
                active_window.end_time.unwrap_or_else(|| "-".to_string())
            );
        }
    }

    if let Some(next_window) = availability.next_window {
        if let Some(label) = next_window.label {
            let mode = next_window
                .mode
                .unwrap_or_else(|| "UNKNOWN".to_string())
                .to_uppercase();
            println!(
                "  {} {} ({})",
                format::styled_dimmed("Next window:"),
                label,
                format::color_mode(&mode)
            );
        }
    }

    if let Some(next_transition) = availability.next_transition_at {
        if let Ok(next_at) = next_transition.parse::<DateTime<Utc>>() {
            println!(
                "  {} {}",
                format::styled_dimmed("Next change:"),
                next_at.format("%a %b %-d %l:%M %p UTC")
            );
        }
    }

    println!();
    Ok(())
}
