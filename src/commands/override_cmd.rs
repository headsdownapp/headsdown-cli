use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const ACTIVE_OVERRIDE_QUERY: &str = r#"
query ActiveAvailabilityOverride {
    activeAvailabilityOverride {
        id
        mode
        reason
        source
        expiresAt
        cancelledAt
        expiredAt
        insertedAt
        updatedAt
    }
}
"#;

const CREATE_OVERRIDE_MUTATION: &str = r#"
mutation CreateAvailabilityOverride($input: AvailabilityOverrideInput!) {
    createAvailabilityOverride(input: $input) {
        id
        mode
        reason
        source
        expiresAt
        cancelledAt
        expiredAt
        insertedAt
        updatedAt
    }
}
"#;

const CANCEL_OVERRIDE_MUTATION: &str = r#"
mutation CancelAvailabilityOverride($id: ID!, $reason: String, $source: String) {
    cancelAvailabilityOverride(id: $id, reason: $reason, source: $source) {
        id
        mode
        reason
        source
        expiresAt
        cancelledAt
        expiredAt
        insertedAt
        updatedAt
    }
}
"#;

#[derive(Deserialize)]
struct ActiveOverrideResponse {
    #[serde(rename = "activeAvailabilityOverride")]
    active_availability_override: Option<AvailabilityOverride>,
}

#[derive(Deserialize)]
struct CreateOverrideResponse {
    #[serde(rename = "createAvailabilityOverride")]
    create_availability_override: AvailabilityOverride,
}

#[derive(Deserialize)]
struct CancelOverrideResponse {
    #[serde(rename = "cancelAvailabilityOverride")]
    cancel_availability_override: AvailabilityOverride,
}

#[derive(Deserialize, Serialize, Clone)]
struct AvailabilityOverride {
    id: String,
    mode: String,
    #[serde(rename = "expiresAt")]
    expires_at: Option<String>,
    #[serde(rename = "cancelledAt")]
    cancelled_at: Option<String>,
}

pub async fn get(api_url: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);
    let data: ActiveOverrideResponse = client.execute_typed(ACTIVE_OVERRIDE_QUERY, None).await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data.active_availability_override)?
        );
        return Ok(());
    }

    println!();
    let current = if let Some(value) = data.active_availability_override {
        value
    } else {
        println!("  {}", format::styled_dimmed("No active override."));
        println!();
        return Ok(());
    };

    println!("  {}", format::styled_bold("Active Override"));
    println!("  {} {}", format::styled_dimmed("ID:"), current.id);
    println!("  {} {}", format::styled_dimmed("Mode:"), current.mode);
    println!(
        "  {} {}",
        format::styled_dimmed("Expires:"),
        current.expires_at.unwrap_or_else(|| "-".to_string())
    );
    println!();
    Ok(())
}

pub async fn set(
    api_url: &str,
    mode: Option<String>,
    duration_minutes: Option<i32>,
    expires_at: Option<String>,
    reason: Option<String>,
    json: bool,
) -> Result<()> {
    let mode = mode.ok_or_else(|| anyhow::anyhow!("--mode is required for override set"))?;

    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);
    let data: CreateOverrideResponse = client
        .execute_typed(
            CREATE_OVERRIDE_MUTATION,
            Some(serde_json::json!({
                "input": {
                    "mode": mode.to_uppercase(),
                    "durationMinutes": duration_minutes,
                    "expiresAt": expires_at,
                    "reason": reason,
                    "source": "hd",
                }
            })),
        )
        .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data.create_availability_override)?
        );
        return Ok(());
    }

    println!();
    println!("  {} Override set", format::styled_green_bold("✓"));
    println!(
        "  {} {}",
        format::styled_dimmed("Mode:"),
        data.create_availability_override.mode
    );
    println!();
    Ok(())
}

pub async fn clear(
    api_url: &str,
    id: Option<String>,
    reason: Option<String>,
    json: bool,
) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let override_id = if let Some(value) = id {
        value
    } else {
        let data: ActiveOverrideResponse =
            client.execute_typed(ACTIVE_OVERRIDE_QUERY, None).await?;
        let active = if let Some(value) = data.active_availability_override {
            value
        } else {
            if json {
                println!("{}", serde_json::json!({ "override": null }));
            } else {
                println!();
                println!(
                    "  {}",
                    format::styled_dimmed("No active override to clear.")
                );
                println!();
            }
            return Ok(());
        };
        active.id
    };

    if override_id.is_empty() {
        bail!("Override id is required");
    }

    let data: CancelOverrideResponse = client
        .execute_typed(
            CANCEL_OVERRIDE_MUTATION,
            Some(serde_json::json!({
                "id": override_id,
                "reason": reason,
                "source": "hd",
            })),
        )
        .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data.cancel_availability_override)?
        );
        return Ok(());
    }

    println!();
    println!("  {} Override cleared", format::styled_green_bold("✓"));
    println!();
    Ok(())
}
