use anyhow::Result;

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

pub async fn run(api_url: &str, json: bool) -> Result<()> {
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

    Ok(())
}
