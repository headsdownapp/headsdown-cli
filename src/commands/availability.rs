use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::auth;
use crate::client::GraphQLClient;
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

pub async fn run(api_url: &str, at: Option<String>, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let variables = serde_json::json!({ "at": at });
    let data = client.execute(AVAILABILITY_QUERY, Some(variables)).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data["availability"])?);
        return Ok(());
    }

    let availability = &data["availability"];

    println!();
    println!("  {}", format::styled_bold("Availability"));
    println!();

    if let Some(in_hours) = availability["inReachableHours"].as_bool() {
        let state = if in_hours {
            "Reachable now"
        } else {
            "Not reachable now"
        };
        println!("  {} {}", format::styled_dimmed("State:"), state);
    }

    if let Some(label) = availability["activeWindow"]["label"].as_str() {
        let mode = availability["activeWindow"]["mode"]
            .as_str()
            .unwrap_or("UNKNOWN")
            .to_uppercase();
        let days = availability["activeWindow"]["days"].as_str().unwrap_or("-");
        let start = availability["activeWindow"]["startTime"]
            .as_str()
            .unwrap_or("-");
        let end = availability["activeWindow"]["endTime"]
            .as_str()
            .unwrap_or("-");
        println!(
            "  {} {} ({})",
            format::styled_dimmed("Active:"),
            label,
            format::color_mode(&mode)
        );
        println!(
            "  {} {} {}-{}",
            format::styled_dimmed("Hours:"),
            days,
            start,
            end
        );
    }

    if let Some(label) = availability["nextWindow"]["label"].as_str() {
        let mode = availability["nextWindow"]["mode"]
            .as_str()
            .unwrap_or("UNKNOWN")
            .to_uppercase();
        println!(
            "  {} {} ({})",
            format::styled_dimmed("Next window:"),
            label,
            format::color_mode(&mode)
        );
    }

    if let Some(next_transition) = availability["nextTransitionAt"].as_str() {
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
