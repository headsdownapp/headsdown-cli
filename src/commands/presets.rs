use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const PRESETS_QUERY: &str = r#"
query {
    presets {
        id
        name
        status
        statusEmoji
        statusText
        duration
        insertedAt
        updatedAt
    }
}
"#;

const APPLY_PRESET_MUTATION: &str = r#"
mutation ApplyPreset($id: ID!) {
    applyPreset(id: $id) {
        id
        mode
        status
        statusEmoji
        statusText
        autoRespond
        lock
        duration
        ruleSetType
        ruleSetParams
        expiresAt
        insertedAt
    }
}
"#;

#[derive(Deserialize)]
struct PresetsResponse {
    presets: Vec<Preset>,
}

#[derive(Deserialize, Serialize, Clone)]
struct Preset {
    id: String,
    name: String,
    #[serde(rename = "statusEmoji")]
    status_emoji: Option<String>,
    #[serde(rename = "statusText")]
    status_text: Option<String>,
    duration: Option<i64>,
}

#[derive(Deserialize)]
struct ApplyPresetResponse {
    #[serde(rename = "applyPreset")]
    apply_preset: AppliedPreset,
}

#[derive(Deserialize, Serialize)]
struct AppliedPreset {
    id: String,
    mode: String,
    #[serde(rename = "expiresAt")]
    expires_at: Option<String>,
}

pub async fn list(api_url: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let data: PresetsResponse = client.execute_typed(PRESETS_QUERY, None).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data.presets)?);
        return Ok(());
    }

    if data.presets.is_empty() {
        println!();
        println!("  No presets configured.");
        println!();
        return Ok(());
    }

    println!();
    println!("  {}", format::styled_bold("Available Presets"));
    println!();

    for preset in &data.presets {
        print_preset(preset);
    }

    println!();
    println!(
        "  {} {}",
        format::styled_dimmed("Tip:"),
        format::styled_dimmed("Activate with: hd preset \"Preset Name\"")
    );
    println!();

    Ok(())
}

pub async fn activate(api_url: &str, name_or_id: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let data: PresetsResponse = client.execute_typed(PRESETS_QUERY, None).await?;

    let preset = data
        .presets
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case(name_or_id) || p.id == name_or_id)
        .cloned()
        .ok_or_else(|| {
            let names: Vec<String> = data.presets.iter().map(|p| p.name.clone()).collect();
            anyhow::anyhow!(
                "Preset '{}' not found. Available: {}",
                name_or_id,
                names.join(", ")
            )
        })?;

    let variables = serde_json::json!({ "id": preset.id });
    let applied: ApplyPresetResponse = client
        .execute_typed(APPLY_PRESET_MUTATION, Some(variables))
        .await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&applied.apply_preset)?);
        return Ok(());
    }

    let mode = applied.apply_preset.mode.to_uppercase();

    println!();
    print!(
        "  {} Applied preset \"{}\" - now {}",
        format::styled_green_bold("✓"),
        format::styled_bold(&preset.name),
        format::color_mode(&mode)
    );

    if let Some(expires_str) = applied.apply_preset.expires_at {
        if let Ok(expires_at) = expires_str.parse::<chrono::DateTime<chrono::Utc>>() {
            let remaining = expires_at.signed_duration_since(chrono::Utc::now());
            if remaining.num_minutes() > 0 {
                print!(" for {}", format::format_duration(remaining.num_minutes()));
            }
        }
    }

    println!();
    println!();

    Ok(())
}

fn print_preset(preset: &Preset) {
    let display_name = if let Some(emoji) = &preset.status_emoji {
        format!("{} {}", emoji, preset.name)
    } else {
        preset.name.clone()
    };

    print!(
        "  {} {}",
        format::styled_dimmed("•"),
        format::styled_bold(&display_name)
    );
    if let Some(duration) = preset.duration {
        print!(" ({})", format::format_duration(duration));
    }
    println!();

    if let Some(status_text) = &preset.status_text {
        if !status_text.is_empty() {
            println!(
                "    {}",
                format::styled_dimmed(&format!("\"{}\"", status_text))
            );
        }
    }
    println!(
        "    {} {}",
        format::styled_dimmed("ID:"),
        format::styled_dimmed(&preset.id)
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn preset_lookup_is_case_insensitive() {
        let list = serde_json::json!([
            {"id":"1","name":"Focus"},
            {"id":"2","name":"Meetings"}
        ]);
        let presets = list.as_array().unwrap();

        let found = presets
            .iter()
            .find(|p| p["name"].as_str().unwrap().eq_ignore_ascii_case("focus"))
            .unwrap();
        assert_eq!(found["id"], "1");
    }
}
