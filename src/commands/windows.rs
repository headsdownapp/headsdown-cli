use anyhow::{bail, Result};
use serde_json::Value;

use crate::auth;
use crate::client::GraphQLClient;
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

pub async fn list(api_url: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);
    let data = client.execute(WINDOWS_QUERY, None).await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data["reachabilityWindows"])?
        );
        return Ok(());
    }

    let windows = data["reachabilityWindows"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No reachability windows found"))?;

    println!();
    println!("  {}", format::styled_bold("Reachability Windows"));
    println!();

    if windows.is_empty() {
        println!("  {}", format::styled_dimmed("No windows configured"));
        println!();
        return Ok(());
    }

    for window in windows {
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
    let data = client
        .execute(CREATE_WINDOW_MUTATION, Some(variables))
        .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data["createReachabilityWindow"])?
        );
        return Ok(());
    }

    println!();
    println!("  {} Window created", format::styled_green_bold("✓"));
    println!();
    print_window(&data["createReachabilityWindow"]);
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
    let data = client
        .execute(UPDATE_WINDOW_MUTATION, Some(variables))
        .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data["updateReachabilityWindow"])?
        );
        return Ok(());
    }

    println!();
    println!("  {} Window updated", format::styled_green_bold("✓"));
    println!();
    print_window(&data["updateReachabilityWindow"]);
    Ok(())
}

pub async fn delete(api_url: &str, id: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let variables = serde_json::json!({ "id": id });
    let data = client
        .execute(DELETE_WINDOW_MUTATION, Some(variables))
        .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data["deleteReachabilityWindow"])?
        );
        return Ok(());
    }

    let window = &data["deleteReachabilityWindow"];
    let label = window["label"].as_str().unwrap_or("Unnamed");

    println!();
    println!(
        "  {} Deleted window {}",
        format::styled_green_bold("✓"),
        format::styled_bold(label)
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
        input["days"] = serde_json::json!(days);
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

fn print_window(window: &Value) {
    let id = window["id"].as_str().unwrap_or("-");
    let label = window["label"].as_str().unwrap_or("Unnamed");
    let mode = window["mode"].as_str().unwrap_or("UNKNOWN").to_uppercase();
    let days = window["days"].as_str().unwrap_or("-");
    let start = window["startTime"].as_str().unwrap_or("-");
    let end = window["endTime"].as_str().unwrap_or("-");
    let policy = window["alertsPolicy"].as_str().unwrap_or("-");
    let priority = window["priority"].as_i64().unwrap_or_default();
    let auto_activate = window["autoActivate"].as_bool().unwrap_or(false);
    let status = window["status"].as_bool().unwrap_or(false);
    let emoji = window["statusEmoji"].as_str().unwrap_or("");
    let status_text = window["statusText"].as_str().unwrap_or("");

    println!(
        "  {} {} ({})",
        format::styled_dimmed("•"),
        format::styled_bold(label),
        format::color_mode(&mode)
    );
    println!(
        "    {} {} {}-{}",
        format::styled_dimmed("Window:"),
        days,
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
        priority,
        format::styled_dimmed("Auto:"),
        auto_activate,
        format::styled_dimmed("Status:"),
        status
    );
    if !emoji.is_empty() || !status_text.is_empty() {
        println!(
            "    {} {} {}",
            format::styled_dimmed("Message:"),
            emoji,
            status_text
        );
    }
    println!(
        "    {} {}",
        format::styled_dimmed("ID:"),
        format::styled_dimmed(id)
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
}
