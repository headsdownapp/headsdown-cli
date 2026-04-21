use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::auth;
use crate::client::GraphQLClient;
use crate::contract::availability::{format_days, DaysField};
use crate::format;

const WINDOWS_QUERY: &str = r#"
query {
    reachabilityWindows {
        id
        label
        mode
        days
        startTime
        endTime
        alertsPolicy
        autoActivate
        priority
        status
        statusEmoji
        statusText
        snooze
    }
}
"#;

const CREATE_WINDOW_MUTATION: &str = r#"
mutation CreateReachabilityWindow($input: ReachabilityWindowInput!) {
    createReachabilityWindow(input: $input) {
        id
        label
        mode
        days
        startTime
        endTime
        alertsPolicy
        autoActivate
        priority
        status
        statusEmoji
        statusText
        snooze
    }
}
"#;

const UPDATE_WINDOW_MUTATION: &str = r#"
mutation UpdateReachabilityWindow($id: ID!, $input: ReachabilityWindowUpdateInput!) {
    updateReachabilityWindow(id: $id, input: $input) {
        id
        label
        mode
        days
        startTime
        endTime
        alertsPolicy
        autoActivate
        priority
        status
        statusEmoji
        statusText
        snooze
    }
}
"#;

const DELETE_WINDOW_MUTATION: &str = r#"
mutation DeleteReachabilityWindow($id: ID!) {
    deleteReachabilityWindow(id: $id) {
        id
        label
        mode
        days
        startTime
        endTime
        alertsPolicy
        autoActivate
        priority
        status
        statusEmoji
        statusText
        snooze
    }
}
"#;

#[derive(Clone, Debug, Default)]
pub struct WindowInputArgs {
    pub label: Option<String>,
    pub mode: Option<String>,
    pub days: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub alerts_policy: Option<String>,
    pub priority: Option<i32>,
    pub auto_activate: Option<bool>,
    pub snooze: Option<bool>,
    pub status: Option<bool>,
    pub status_emoji: Option<String>,
    pub status_text: Option<String>,
}

#[derive(Deserialize)]
struct WindowsResponse {
    #[serde(rename = "reachabilityWindows")]
    reachability_windows: Vec<ReachabilityWindow>,
}

#[derive(Deserialize)]
struct WindowMutationResponse {
    #[serde(rename = "createReachabilityWindow")]
    create_reachability_window: Option<ReachabilityWindow>,
    #[serde(rename = "updateReachabilityWindow")]
    update_reachability_window: Option<ReachabilityWindow>,
    #[serde(rename = "deleteReachabilityWindow")]
    delete_reachability_window: Option<ReachabilityWindow>,
}

#[derive(Deserialize, Serialize)]
struct ReachabilityWindow {
    id: String,
    label: String,
    mode: String,
    days: Option<DaysField>,
    #[serde(rename = "startTime")]
    start_time: Option<String>,
    #[serde(rename = "endTime")]
    end_time: Option<String>,
    #[serde(rename = "alertsPolicy")]
    alerts_policy: Option<String>,
    #[serde(rename = "autoActivate")]
    auto_activate: Option<bool>,
    priority: Option<i64>,
    status: Option<bool>,
    #[serde(rename = "statusEmoji")]
    status_emoji: Option<String>,
    #[serde(rename = "statusText")]
    status_text: Option<String>,
}

pub async fn list(api_url: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);
    let data: WindowsResponse = client.execute_typed(WINDOWS_QUERY, None).await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data.reachability_windows)?
        );
        return Ok(());
    }

    println!();
    println!("  {}", format::styled_bold("Reachability Windows"));
    println!();

    if data.reachability_windows.is_empty() {
        println!("  {}", format::styled_dimmed("No windows configured"));
        println!();
        return Ok(());
    }

    for window in &data.reachability_windows {
        print_window(window);
    }

    Ok(())
}

pub async fn create(api_url: &str, args: WindowInputArgs, json: bool) -> Result<()> {
    require_create_fields(&args)?;

    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let input = build_input(args);
    let variables = serde_json::json!({ "input": input });
    let data: WindowMutationResponse = client
        .execute_typed(CREATE_WINDOW_MUTATION, Some(variables))
        .await?;
    let window = data
        .create_reachability_window
        .ok_or_else(|| anyhow!("Missing createReachabilityWindow in response"))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&window)?);
        return Ok(());
    }

    println!();
    println!("  {} Window created", format::styled_green_bold("✓"));
    println!();
    print_window(&window);
    Ok(())
}

pub async fn update(api_url: &str, id: &str, args: WindowInputArgs, json: bool) -> Result<()> {
    if all_fields_empty(&args) {
        bail!("No updates provided. Pass at least one field to update.");
    }

    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let input = build_input(args);
    let variables = serde_json::json!({ "id": id, "input": input });
    let data: WindowMutationResponse = client
        .execute_typed(UPDATE_WINDOW_MUTATION, Some(variables))
        .await?;
    let window = data
        .update_reachability_window
        .ok_or_else(|| anyhow!("Missing updateReachabilityWindow in response"))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&window)?);
        return Ok(());
    }

    println!();
    println!("  {} Window updated", format::styled_green_bold("✓"));
    println!();
    print_window(&window);
    Ok(())
}

pub async fn delete(api_url: &str, id: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let variables = serde_json::json!({ "id": id });
    let data: WindowMutationResponse = client
        .execute_typed(DELETE_WINDOW_MUTATION, Some(variables))
        .await?;
    let window = data
        .delete_reachability_window
        .ok_or_else(|| anyhow!("Missing deleteReachabilityWindow in response"))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&window)?);
        return Ok(());
    }

    println!();
    println!(
        "  {} Deleted window {}",
        format::styled_green_bold("✓"),
        format::styled_bold(&window.label)
    );
    println!(
        "  {} {}",
        format::styled_dimmed("ID:"),
        format::styled_dimmed(id)
    );
    println!();

    Ok(())
}

fn require_create_fields(args: &WindowInputArgs) -> Result<()> {
    if args.label.is_none()
        || args.mode.is_none()
        || args.days.is_none()
        || args.start.is_none()
        || args.end.is_none()
    {
        bail!("Create requires --label, --mode, --days, --start, and --end.");
    }
    Ok(())
}

fn all_fields_empty(args: &WindowInputArgs) -> bool {
    args.label.is_none()
        && args.mode.is_none()
        && args.days.is_none()
        && args.start.is_none()
        && args.end.is_none()
        && args.alerts_policy.is_none()
        && args.priority.is_none()
        && args.auto_activate.is_none()
        && args.snooze.is_none()
        && args.status.is_none()
        && args.status_emoji.is_none()
        && args.status_text.is_none()
}

fn build_input(args: WindowInputArgs) -> Value {
    let mut input = serde_json::json!({});

    if let Some(label) = args.label {
        input["label"] = serde_json::json!(label);
    }
    if let Some(mode) = args.mode {
        input["mode"] = serde_json::json!(normalize_mode(&mode));
    }
    if let Some(days) = args.days {
        input["days"] = serde_json::json!(normalize_days_input(&days));
    }
    if let Some(start) = args.start {
        input["startTime"] = serde_json::json!(start);
    }
    if let Some(end) = args.end {
        input["endTime"] = serde_json::json!(end);
    }
    if let Some(policy) = args.alerts_policy {
        input["alertsPolicy"] = serde_json::json!(normalize_alerts_policy(&policy));
    }
    if let Some(priority) = args.priority {
        input["priority"] = serde_json::json!(priority);
    }
    if let Some(auto_activate) = args.auto_activate {
        input["autoActivate"] = serde_json::json!(auto_activate);
    }
    if let Some(snooze) = args.snooze {
        input["snooze"] = serde_json::json!(snooze);
    }
    if let Some(status) = args.status {
        input["status"] = serde_json::json!(status);
    }
    if let Some(status_emoji) = args.status_emoji {
        input["statusEmoji"] = serde_json::json!(status_emoji);
    }
    if let Some(status_text) = args.status_text {
        input["statusText"] = serde_json::json!(status_text);
    }

    input
}

fn normalize_mode(mode: &str) -> String {
    mode.trim().replace('-', "_").to_uppercase()
}

fn normalize_alerts_policy(policy: &str) -> String {
    policy.trim().replace('-', "_").to_uppercase()
}

fn normalize_days_input(input: &str) -> Vec<String> {
    let normalized = input.trim();

    if normalized.contains('-') {
        let parts: Vec<&str> = normalized.split('-').collect();
        if parts.len() == 2 {
            let start = normalize_day(parts[0]);
            let end = normalize_day(parts[1]);
            let ordered = vec![
                "MONDAY",
                "TUESDAY",
                "WEDNESDAY",
                "THURSDAY",
                "FRIDAY",
                "SATURDAY",
                "SUNDAY",
            ];
            if let (Some(start_idx), Some(end_idx)) = (
                ordered.iter().position(|d| d == &start),
                ordered.iter().position(|d| d == &end),
            ) {
                return if start_idx <= end_idx {
                    ordered[start_idx..=end_idx]
                        .iter()
                        .map(|d| d.to_string())
                        .collect()
                } else {
                    ordered[start_idx..]
                        .iter()
                        .chain(ordered[..=end_idx].iter())
                        .map(|d| d.to_string())
                        .collect()
                };
            }
        }
    }

    normalized
        .split(',')
        .map(normalize_day)
        .collect::<Vec<String>>()
}

fn normalize_day(day: &str) -> String {
    match day.trim().to_lowercase().as_str() {
        "mon" | "monday" => "MONDAY".to_string(),
        "tue" | "tues" | "tuesday" => "TUESDAY".to_string(),
        "wed" | "wednesday" => "WEDNESDAY".to_string(),
        "thu" | "thur" | "thurs" | "thursday" => "THURSDAY".to_string(),
        "fri" | "friday" => "FRIDAY".to_string(),
        "sat" | "saturday" => "SATURDAY".to_string(),
        "sun" | "sunday" => "SUNDAY".to_string(),
        other => other.replace('-', "_").to_uppercase(),
    }
}

fn print_window(window: &ReachabilityWindow) {
    let mode = window.mode.to_uppercase();
    let days = format_days(window.days.as_ref());
    let start = window.start_time.clone().unwrap_or_else(|| "-".to_string());
    let end = window.end_time.clone().unwrap_or_else(|| "-".to_string());
    let policy = window
        .alerts_policy
        .clone()
        .unwrap_or_else(|| "-".to_string());

    println!(
        "  {} {} ({})",
        format::styled_dimmed("•"),
        format::styled_bold(&window.label),
        format::color_mode(&mode)
    );
    println!(
        "    {} {} {}-{}",
        format::styled_dimmed("Window:"),
        &days,
        start,
        end
    );
    println!(
        "    {} {}",
        format::styled_dimmed("Alerts:"),
        policy.to_lowercase().replace('_', " ")
    );
    println!(
        "    {} {}  {} {}  {} {}",
        format::styled_dimmed("Priority:"),
        window.priority.unwrap_or_default(),
        format::styled_dimmed("Auto:"),
        window.auto_activate.unwrap_or(false),
        format::styled_dimmed("Status:"),
        window.status.unwrap_or(false)
    );
    if window.status_emoji.as_deref().unwrap_or("") != ""
        || window.status_text.as_deref().unwrap_or("") != ""
    {
        println!(
            "    {} {} {}",
            format::styled_dimmed("Message:"),
            window.status_emoji.clone().unwrap_or_default(),
            window.status_text.clone().unwrap_or_default()
        );
    }
    println!(
        "    {} {}",
        format::styled_dimmed("ID:"),
        format::styled_dimmed(&window.id)
    );
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_input_normalizes_enums_and_maps_fields() {
        let args = WindowInputArgs {
            label: Some("Focus".to_string()),
            mode: Some("busy".to_string()),
            days: Some("Mon-Fri".to_string()),
            start: Some("09:00:00".to_string()),
            end: Some("17:00:00".to_string()),
            alerts_policy: Some("do_not_disturb".to_string()),
            priority: Some(10),
            auto_activate: Some(true),
            snooze: Some(false),
            status: Some(true),
            status_emoji: Some("🎧".to_string()),
            status_text: Some("Deep work".to_string()),
        };

        let input = build_input(args);

        assert_eq!(input["mode"], "BUSY");
        assert_eq!(input["alertsPolicy"], "DO_NOT_DISTURB");
        assert_eq!(input["startTime"], "09:00:00");
        assert_eq!(input["endTime"], "17:00:00");
        assert_eq!(input["statusEmoji"], "🎧");
    }

    #[test]
    fn create_requires_core_fields() {
        let args = WindowInputArgs::default();
        assert!(require_create_fields(&args).is_err());

        let args = WindowInputArgs {
            label: Some("Focus".to_string()),
            mode: Some("busy".to_string()),
            days: Some("Mon-Fri".to_string()),
            start: Some("09:00:00".to_string()),
            end: Some("17:00:00".to_string()),
            ..WindowInputArgs::default()
        };
        assert!(require_create_fields(&args).is_ok());
    }

    #[test]
    fn all_fields_empty_detects_changes() {
        assert!(all_fields_empty(&WindowInputArgs::default()));
        assert!(!all_fields_empty(&WindowInputArgs {
            priority: Some(5),
            ..WindowInputArgs::default()
        }));
    }

    #[test]
    fn normalize_days_input_supports_ranges_and_lists() {
        assert_eq!(normalize_days_input("Mon-Fri").len(), 5);
        assert_eq!(
            normalize_days_input("Mon,Wed,Fri"),
            vec!["MONDAY", "WEDNESDAY", "FRIDAY"]
        );
    }

    #[test]
    fn format_days_reads_array_shape() {
        let value = DaysField::List(vec!["MONDAY".to_string(), "TUESDAY".to_string()]);
        assert_eq!(format_days(Some(&value)), "monday,tuesday");
    }
}
