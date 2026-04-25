use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const VERDICT_SETTINGS_QUERY: &str = r#"
query {
    verdictSettings {
        id
        thresholds {
            online {
                maxFiles
                maxEstimatedMinutes
            }
            busy {
                maxFiles
                maxEstimatedMinutes
            }
            limited {
                maxFiles
                maxEstimatedMinutes
            }
            offline {
                maxFiles
                maxEstimatedMinutes
            }
        }
        defaultWrapUpMode
        wrapUpThresholdMinutes
        updatedAt
    }
}
"#;

const UPDATE_VERDICT_SETTINGS_MUTATION: &str = r#"
mutation UpdateVerdictSettings($thresholds: VerdictModeThresholdsInput, $defaultWrapUpMode: WrapUpMode, $wrapUpThresholdMinutes: Int) {
    updateVerdictSettings(thresholds: $thresholds, defaultWrapUpMode: $defaultWrapUpMode, wrapUpThresholdMinutes: $wrapUpThresholdMinutes) {
        id
        thresholds {
            online {
                maxFiles
                maxEstimatedMinutes
            }
            busy {
                maxFiles
                maxEstimatedMinutes
            }
            limited {
                maxFiles
                maxEstimatedMinutes
            }
            offline {
                maxFiles
                maxEstimatedMinutes
            }
        }
        defaultWrapUpMode
        wrapUpThresholdMinutes
        updatedAt
    }
}
"#;

#[derive(Deserialize)]
struct VerdictSettingsResponse {
    #[serde(rename = "verdictSettings")]
    verdict_settings: VerdictSettings,
}

#[derive(Deserialize)]
struct UpdateVerdictSettingsResponse {
    #[serde(rename = "updateVerdictSettings")]
    update_verdict_settings: VerdictSettings,
}

#[derive(Deserialize, Serialize)]
struct VerdictSettings {
    id: String,
    thresholds: Value,
    #[serde(rename = "defaultWrapUpMode")]
    default_wrap_up_mode: String,
    #[serde(rename = "wrapUpThresholdMinutes")]
    wrap_up_threshold_minutes: i64,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

pub async fn get(api_url: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);
    let data: VerdictSettingsResponse = client.execute_typed(VERDICT_SETTINGS_QUERY, None).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data.verdict_settings)?);
        return Ok(());
    }

    let settings = data.verdict_settings;
    println!();
    println!("  {}", format::styled_bold("Verdict settings"));
    println!();
    println!("  {} {}", format::styled_dimmed("ID:"), settings.id);
    println!(
        "  {} {}",
        format::styled_dimmed("Default wrap-up mode:"),
        settings.default_wrap_up_mode
    );
    println!(
        "  {} {} min",
        format::styled_dimmed("Wrap-up threshold:"),
        settings.wrap_up_threshold_minutes
    );
    println!("  {}", format::styled_dimmed("Thresholds:"));
    println!("{}", serde_json::to_string_pretty(&settings.thresholds)?);
    println!();
    Ok(())
}

pub async fn set(
    api_url: &str,
    thresholds: Option<&str>,
    default_wrap_up_mode: Option<&str>,
    wrap_up_threshold_minutes: Option<i32>,
    json: bool,
) -> Result<()> {
    if thresholds.is_none() && default_wrap_up_mode.is_none() && wrap_up_threshold_minutes.is_none()
    {
        bail!("No updates provided. Pass at least one of --thresholds, --default-wrap-up-mode, or --wrap-up-threshold-minutes.");
    }

    let parsed_thresholds: Option<Value> = thresholds
        .map(|raw| serde_json::from_str(raw).context("thresholds must be valid JSON"))
        .transpose()?;

    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);
    let data: UpdateVerdictSettingsResponse = client
        .execute_typed(
            UPDATE_VERDICT_SETTINGS_MUTATION,
            Some(serde_json::json!({
                "thresholds": parsed_thresholds,
                "defaultWrapUpMode": default_wrap_up_mode.map(|v| v.to_uppercase()),
                "wrapUpThresholdMinutes": wrap_up_threshold_minutes,
            })),
        )
        .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data.update_verdict_settings)?
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
        serde_json::to_string_pretty(&data.update_verdict_settings)?
    );
    println!();
    Ok(())
}
