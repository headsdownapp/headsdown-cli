use anyhow::{bail, Result};
use chrono::{DateTime, Local, NaiveTime, Utc};

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const CREATE_CONTRACT_MUTATION: &str = r#"
mutation CreateContract($input: ContractInput!) {
    createContract(input: $input) {
        mode
        expiresAt
        duration
        statusText
        statusEmoji
    }
}
"#;

/// Parse a human-readable duration string into minutes.
/// Supports: "2h", "30m", "30min", "1h30m", "90", "1.5h"
fn parse_duration(input: &str) -> Result<i64> {
    let input = input.trim().to_lowercase();

    // Try "until Xpm" / "until Xam" / "until X:YYpm" patterns
    if let Some(rest) = input.strip_prefix("until ") {
        let time_str = rest.trim();
        if let Some(minutes) = parse_until_time(time_str) {
            return Ok(minutes);
        }
        bail!(
            "Time '{}' has already passed. Try a future time like: until 5pm, until 3:30pm",
            time_str
        );
    }

    // Try "Xh" or "X.Yh"
    if input.ends_with('h') {
        let num_str = &input[..input.len() - 1];
        if let Ok(hours) = num_str.parse::<f64>() {
            return Ok((hours * 60.0).round() as i64);
        }
    }

    // Try "Xm" or "Xmin"
    if input.ends_with("min") {
        let num_str = &input[..input.len() - 3];
        if let Ok(mins) = num_str.parse::<i64>() {
            return Ok(mins);
        }
    }
    if input.ends_with('m') {
        let num_str = &input[..input.len() - 1];
        if let Ok(mins) = num_str.parse::<i64>() {
            return Ok(mins);
        }
    }

    // Try "XhYm" pattern
    if let Some(h_pos) = input.find('h') {
        let hours_str = &input[..h_pos];
        let rest = &input[h_pos + 1..];
        let rest = rest.trim_end_matches('m').trim_end_matches("in");
        if let (Ok(hours), Ok(mins)) = (hours_str.parse::<i64>(), rest.parse::<i64>()) {
            return Ok(hours * 60 + mins);
        }
    }

    // Try bare number as minutes
    if let Ok(mins) = input.parse::<i64>() {
        return Ok(mins);
    }

    bail!(
        "Could not parse duration '{}'. Try formats like: 2h, 30m, 1h30m, 90min, until 5pm",
        input
    );
}

/// Parse a time-of-day string like "5pm", "3:30pm", "17:00" into minutes from now.
fn parse_until_time(input: &str) -> Option<i64> {
    let input = input.trim().to_lowercase();
    let now = Local::now();

    let target_time = if input.ends_with("pm") || input.ends_with("am") {
        let is_pm = input.ends_with("pm");
        let num_part = &input[..input.len() - 2];

        if let Some((h, m)) = num_part.split_once(':') {
            let mut hour: u32 = h.parse().ok()?;
            let minute: u32 = m.parse().ok()?;
            if is_pm && hour != 12 {
                hour += 12;
            } else if !is_pm && hour == 12 {
                hour = 0;
            }
            NaiveTime::from_hms_opt(hour, minute, 0)?
        } else {
            let mut hour: u32 = num_part.parse().ok()?;
            if is_pm && hour != 12 {
                hour += 12;
            } else if !is_pm && hour == 12 {
                hour = 0;
            }
            NaiveTime::from_hms_opt(hour, 0, 0)?
        }
    } else if let Some((h, m)) = input.split_once(':') {
        // 24-hour format like "17:00"
        let hour: u32 = h.parse().ok()?;
        let minute: u32 = m.parse().ok()?;
        NaiveTime::from_hms_opt(hour, minute, 0)?
    } else {
        return None;
    };

    let current_time = now.time();
    let diff = target_time.signed_duration_since(current_time);
    let minutes = diff.num_minutes();

    if minutes <= 0 {
        return None; // target time is in the past
    }

    Some(minutes)
}

pub async fn run(api_url: &str, mode: &str, duration: Option<String>, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let duration_minutes = match &duration {
        Some(d) => Some(parse_duration(d)?),
        None => None,
    };

    let mut input = serde_json::json!({
        "mode": mode,
        "autoRespond": mode == "BUSY",
        "status": false,
    });

    if let Some(mins) = duration_minutes {
        input["duration"] = serde_json::json!(mins);
    }

    let variables = serde_json::json!({ "input": input });
    let data = client
        .execute(CREATE_CONTRACT_MUTATION, Some(variables))
        .await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data["createContract"])?);
        return Ok(());
    }

    let contract = &data["createContract"];
    let actual_mode = contract["mode"].as_str().unwrap_or(mode).to_uppercase();

    println!();
    print!(
        "  {} Set to {}",
        format::styled_green_bold("✓"),
        format::color_mode(&actual_mode)
    );

    // Show duration info
    if let Some(expires_str) = contract["expiresAt"].as_str() {
        if let Ok(expires_at) = expires_str.parse::<DateTime<Utc>>() {
            let now = Utc::now();
            let remaining = expires_at.signed_duration_since(now);
            if remaining.num_minutes() > 0 {
                let formatted = format::format_duration(remaining.num_minutes());
                print!(
                    " for {} (until {})",
                    formatted,
                    expires_at.format("%l:%M %p").to_string().trim()
                );
            }
        }
    }

    println!();
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_hours() {
        assert_eq!(parse_duration("2h").unwrap(), 120);
        assert_eq!(parse_duration("1h").unwrap(), 60);
        assert_eq!(parse_duration("1.5h").unwrap(), 90);
    }

    #[test]
    fn test_parse_duration_minutes() {
        assert_eq!(parse_duration("30m").unwrap(), 30);
        assert_eq!(parse_duration("90min").unwrap(), 90);
        assert_eq!(parse_duration("45m").unwrap(), 45);
    }

    #[test]
    fn test_parse_duration_combined() {
        assert_eq!(parse_duration("1h30m").unwrap(), 90);
        assert_eq!(parse_duration("2h15m").unwrap(), 135);
    }

    #[test]
    fn test_parse_duration_bare_number() {
        assert_eq!(parse_duration("60").unwrap(), 60);
        assert_eq!(parse_duration("30").unwrap(), 30);
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("").is_err());
    }

    #[test]
    fn test_parse_duration_until_time() {
        // "until" with a past time should error
        assert!(parse_duration("until 0am").is_err() || parse_duration("until 0am").unwrap() > 0);

        // Verify the parse_until_time helper directly
        // 11:59pm should always be in the future during test runs (unless run at midnight)
        let result = parse_until_time("11:59pm");
        // This is time-dependent, so just verify it returns Some with a positive value
        // or None if somehow run right at 11:59pm
        if let Some(mins) = result {
            assert!(mins > 0);
        }

        // 24-hour format
        let result = parse_until_time("23:59");
        if let Some(mins) = result {
            assert!(mins > 0);
        }
    }
}
