use anyhow::{Context, Result};
use serde_json::Value;

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const VERDICT_SETTINGS_QUERY: &str = r#"
query {
    verdictSettings {
        id
        modeThresholds
        updatedAt
    }
}
"#;

const UPDATE_VERDICT_SETTINGS_MUTATION: &str = r#"
mutation UpdateVerdictSettings($modeThresholds: JSON) {
    updateVerdictSettings(modeThresholds: $modeThresholds) {
        id
        modeThresholds
        updatedAt
    }
}
"#;

pub async fn get(api_url: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);
    let data = client.execute(VERDICT_SETTINGS_QUERY, None).await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data["verdictSettings"])?
        );
        return Ok(());
    }

    let settings = &data["verdictSettings"];
    println!();
    println!("  {}", format::styled_bold("Verdict settings"));
    println!();
    println!(
        "  {} {}",
        format::styled_dimmed("ID:"),
        settings["id"].as_str().unwrap_or("-")
    );
    println!("  {}", format::styled_dimmed("Mode thresholds:"));
    println!(
        "{}",
        serde_json::to_string_pretty(&settings["modeThresholds"])?
    );
    println!();
    Ok(())
}

pub async fn set(api_url: &str, mode_thresholds: &str, json: bool) -> Result<()> {
    let parsed: Value =
        serde_json::from_str(mode_thresholds).context("mode_thresholds must be valid JSON")?;

    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);
    let data = client
        .execute(
            UPDATE_VERDICT_SETTINGS_MUTATION,
            Some(serde_json::json!({ "modeThresholds": parsed })),
        )
        .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data["updateVerdictSettings"])?
        );
        return Ok(());
    }

    println!();
    println!(
        "  {} Verdict settings updated",
        format::styled_green_bold("✓")
    );
    println!();
    println!(
        "{}",
        serde_json::to_string_pretty(&data["updateVerdictSettings"]["modeThresholds"])?
    );
    println!();
    Ok(())
}
