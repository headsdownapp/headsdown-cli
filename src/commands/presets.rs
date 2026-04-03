use anyhow::Result;

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const PRESETS_QUERY: &str = r#"
query {
    presets {
        id
        name
        alerts
        presence
        duration
        statusEmoji
        statusText
    }
}
"#;

const APPLY_PRESET_MUTATION: &str = r#"
mutation ApplyPreset($id: ID!) {
    applyPreset(id: $id) {
        mode
        expiresAt
        duration
        statusText
        statusEmoji
    }
}
"#;

pub async fn run(api_url: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let data = client.execute(PRESETS_QUERY, None).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data["presets"])?);
        return Ok(());
    }

    let presets = data["presets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No presets found"))?;

    if presets.is_empty() {
        println!();
        println!("  No presets configured. Create presets in the HeadsDown app.");
        println!();
        return Ok(());
    }

    println!();
    println!("  {}", format::styled_bold("Available Presets"));
    println!();

    for preset in presets {
        let name = preset["name"].as_str().unwrap_or("Unknown");
        let alerts = preset["alerts"].as_str().unwrap_or("");
        let presence = preset["presence"].as_str().unwrap_or("");
        let emoji = preset["statusEmoji"].as_str().unwrap_or("");
        let status_text = preset["statusText"].as_str().unwrap_or("");

        // Format the preset name with any emoji
        let display_name = if !emoji.is_empty() {
            format!("{} {}", emoji, name)
        } else {
            name.to_string()
        };

        print!(
            "  {} {}",
            format::styled_dimmed("•"),
            format::styled_bold(&display_name)
        );

        // Show duration if set
        if let Some(duration) = preset["duration"].as_i64() {
            print!(" ({})", format::format_duration(duration));
        }

        println!();

        // Show details on second line
        let mut details = Vec::new();
        if !alerts.is_empty() {
            details.push(format!("alerts: {}", format_enum_value(alerts)));
        }
        if !presence.is_empty() {
            details.push(format!("presence: {}", format_enum_value(presence)));
        }
        if !status_text.is_empty() {
            details.push(format!("\"{}\"", status_text));
        }
        if !details.is_empty() {
            println!("    {}", format::styled_dimmed(&details.join(" · ")));
        }
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

    // First, list presets to find the matching one
    let data = client.execute(PRESETS_QUERY, None).await?;
    let presets = data["presets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No presets found"))?;

    // Find by name (case-insensitive) or ID
    let preset = presets
        .iter()
        .find(|p| {
            let pname = p["name"].as_str().unwrap_or("");
            let pid = p["id"].as_str().unwrap_or("");
            pname.eq_ignore_ascii_case(name_or_id) || pid == name_or_id
        })
        .ok_or_else(|| {
            let names: Vec<&str> = presets.iter().filter_map(|p| p["name"].as_str()).collect();
            anyhow::anyhow!(
                "Preset '{}' not found. Available: {}",
                name_or_id,
                names.join(", ")
            )
        })?;

    let preset_id = preset["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Preset missing ID"))?;
    let preset_name = preset["name"].as_str().unwrap_or(name_or_id);

    let variables = serde_json::json!({ "id": preset_id });
    let data = client
        .execute(APPLY_PRESET_MUTATION, Some(variables))
        .await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data["applyPreset"])?);
        return Ok(());
    }

    let contract = &data["applyPreset"];
    let mode = contract["mode"]
        .as_str()
        .unwrap_or("UNKNOWN")
        .to_uppercase();

    println!();
    print!(
        "  {} Applied preset \"{}\" - now {}",
        format::styled_green_bold("✓"),
        format::styled_bold(preset_name),
        format::color_mode(&mode)
    );

    // Show duration if present
    if let Some(expires_str) = contract["expiresAt"].as_str() {
        if let Ok(expires_at) = expires_str.parse::<chrono::DateTime<chrono::Utc>>() {
            let now = chrono::Utc::now();
            let remaining = expires_at.signed_duration_since(now);
            if remaining.num_minutes() > 0 {
                print!(" for {}", format::format_duration(remaining.num_minutes()));
            }
        }
    }

    println!();
    println!();

    Ok(())
}

/// Convert SCREAMING_SNAKE_CASE enum values to readable text.
fn format_enum_value(s: &str) -> String {
    s.to_lowercase().replace('_', " ")
}
