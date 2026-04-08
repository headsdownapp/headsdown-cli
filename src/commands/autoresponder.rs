use anyhow::{bail, Result};

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const AUTO_RESPONDER_QUERY: &str = r#"
query {
    autoResponderSettings {
        id
        busyText
        limitedText
        offlineText
        updatedAt
    }
}
"#;

const UPDATE_AUTO_RESPONDER_MUTATION: &str = r#"
mutation UpdateAutoResponderSettings($busyText: String, $limitedText: String, $offlineText: String) {
    updateAutoResponderSettings(busyText: $busyText, limitedText: $limitedText, offlineText: $offlineText) {
        id
        busyText
        limitedText
        offlineText
        updatedAt
    }
}
"#;

pub async fn get(api_url: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);
    let data = client.execute(AUTO_RESPONDER_QUERY, None).await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data["autoResponderSettings"])?
        );
        return Ok(());
    }

    let settings = &data["autoResponderSettings"];
    println!();
    println!("  {}", format::styled_bold("Auto-responder"));
    println!();
    println!(
        "  {} {}",
        format::styled_dimmed("Busy:"),
        settings["busyText"].as_str().unwrap_or("-")
    );
    println!(
        "  {} {}",
        format::styled_dimmed("Limited:"),
        settings["limitedText"].as_str().unwrap_or("-")
    );
    println!(
        "  {} {}",
        format::styled_dimmed("Offline:"),
        settings["offlineText"].as_str().unwrap_or("-")
    );
    println!();
    Ok(())
}

pub async fn set(
    api_url: &str,
    busy_text: Option<String>,
    limited_text: Option<String>,
    offline_text: Option<String>,
    json: bool,
) -> Result<()> {
    if busy_text.is_none() && limited_text.is_none() && offline_text.is_none() {
        bail!("No updates provided. Pass at least one of --busy-text, --limited-text, or --offline-text.");
    }

    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let variables = serde_json::json!({
        "busyText": busy_text,
        "limitedText": limited_text,
        "offlineText": offline_text,
    });
    let data = client
        .execute(UPDATE_AUTO_RESPONDER_MUTATION, Some(variables))
        .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data["updateAutoResponderSettings"])?
        );
        return Ok(());
    }

    println!();
    println!(
        "  {} Auto-responder updated",
        format::styled_green_bold("✓")
    );
    println!();
    let settings = &data["updateAutoResponderSettings"];
    println!(
        "  {} {}",
        format::styled_dimmed("Busy:"),
        settings["busyText"].as_str().unwrap_or("-")
    );
    println!(
        "  {} {}",
        format::styled_dimmed("Limited:"),
        settings["limitedText"].as_str().unwrap_or("-")
    );
    println!(
        "  {} {}",
        format::styled_dimmed("Offline:"),
        settings["offlineText"].as_str().unwrap_or("-")
    );
    println!();
    Ok(())
}
