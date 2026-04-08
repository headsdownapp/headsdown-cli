use anyhow::{bail, Result};
use serde_json::Value;

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
        status
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

const CREATE_PRESET_MUTATION: &str = r#"
mutation CreatePreset($input: PresetInput!) {
    createPreset(input: $input) {
        id
        name
        alerts
        presence
        duration
        status
        statusEmoji
        statusText
    }
}
"#;

const UPDATE_PRESET_MUTATION: &str = r#"
mutation UpdatePreset($id: ID!, $input: PresetInput!) {
    updatePreset(id: $id, input: $input) {
        id
        name
        alerts
        presence
        duration
        status
        statusEmoji
        statusText
    }
}
"#;

const DELETE_PRESET_MUTATION: &str = r#"
mutation DeletePreset($id: ID!) {
    deletePreset(id: $id) {
        id
        name
    }
}
"#;

#[derive(Clone, Debug, Default)]
pub struct PresetInputArgs {
    pub name: Option<String>,
    pub alerts: Option<String>,
    pub presence: Option<String>,
    pub duration: Option<i32>,
    pub status: Option<bool>,
    pub status_emoji: Option<String>,
    pub status_text: Option<String>,
}

pub async fn list(api_url: &str, json: bool) -> Result<()> {
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
        println!("  No presets configured. Create one with `hd presets create`.");
        println!();
        return Ok(());
    }

    println!();
    println!("  {}", format::styled_bold("Available Presets"));
    println!();

    for preset in presets {
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

    let data = client.execute(PRESETS_QUERY, None).await?;
    let presets = data["presets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No presets found"))?;

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

    if let Some(expires_str) = contract["expiresAt"].as_str() {
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

pub async fn create(api_url: &str, args: PresetInputArgs, json: bool) -> Result<()> {
    if args.name.is_none() {
        bail!("Create requires --name.");
    }

    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);
    let input = build_input(args);
    let data = client
        .execute(
            CREATE_PRESET_MUTATION,
            Some(serde_json::json!({ "input": input })),
        )
        .await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data["createPreset"])?);
        return Ok(());
    }

    println!();
    println!("  {} Preset created", format::styled_green_bold("✓"));
    println!();
    print_preset(&data["createPreset"]);
    Ok(())
}

pub async fn update(api_url: &str, id: &str, args: PresetInputArgs, json: bool) -> Result<()> {
    if all_fields_empty(&args) {
        bail!("No updates provided. Pass at least one field to update.");
    }

    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);
    let input = build_input(args);
    let data = client
        .execute(
            UPDATE_PRESET_MUTATION,
            Some(serde_json::json!({ "id": id, "input": input })),
        )
        .await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data["updatePreset"])?);
        return Ok(());
    }

    println!();
    println!("  {} Preset updated", format::styled_green_bold("✓"));
    println!();
    print_preset(&data["updatePreset"]);
    Ok(())
}

pub async fn delete(api_url: &str, id: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);
    let data = client
        .execute(
            DELETE_PRESET_MUTATION,
            Some(serde_json::json!({ "id": id })),
        )
        .await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data["deletePreset"])?);
        return Ok(());
    }

    println!();
    println!(
        "  {} Deleted preset {}",
        format::styled_green_bold("✓"),
        format::styled_bold(data["deletePreset"]["name"].as_str().unwrap_or("Unknown"))
    );
    println!(
        "  {} {}",
        format::styled_dimmed("ID:"),
        format::styled_dimmed(id)
    );
    println!();
    Ok(())
}

fn build_input(args: PresetInputArgs) -> Value {
    let mut input = serde_json::json!({});

    if let Some(name) = args.name {
        input["name"] = serde_json::json!(name);
    }
    if let Some(alerts) = args.alerts {
        input["alerts"] = serde_json::json!(normalize_enum(&alerts));
    }
    if let Some(presence) = args.presence {
        input["presence"] = serde_json::json!(normalize_enum(&presence));
    }
    if let Some(duration) = args.duration {
        input["duration"] = serde_json::json!(duration);
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

fn all_fields_empty(args: &PresetInputArgs) -> bool {
    args.name.is_none()
        && args.alerts.is_none()
        && args.presence.is_none()
        && args.duration.is_none()
        && args.status.is_none()
        && args.status_emoji.is_none()
        && args.status_text.is_none()
}

fn normalize_enum(input: &str) -> String {
    input.trim().replace('-', "_").to_uppercase()
}

fn print_preset(preset: &Value) {
    let id = preset["id"].as_str().unwrap_or("-");
    let name = preset["name"].as_str().unwrap_or("Unknown");
    let alerts = preset["alerts"].as_str().unwrap_or("");
    let presence = preset["presence"].as_str().unwrap_or("");
    let emoji = preset["statusEmoji"].as_str().unwrap_or("");
    let status_text = preset["statusText"].as_str().unwrap_or("");

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
    if let Some(duration) = preset["duration"].as_i64() {
        print!(" ({})", format::format_duration(duration));
    }
    println!();

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
    println!(
        "    {} {}",
        format::styled_dimmed("ID:"),
        format::styled_dimmed(id)
    );
}

fn format_enum_value(s: &str) -> String {
    s.to_lowercase().replace('_', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_enum_maps_to_upper_snake_case() {
        assert_eq!(normalize_enum("do_not_disturb"), "DO_NOT_DISTURB");
        assert_eq!(normalize_enum("take-a-number"), "TAKE_A_NUMBER");
    }

    #[test]
    fn update_requires_at_least_one_field() {
        assert!(all_fields_empty(&PresetInputArgs::default()));
        assert!(!all_fields_empty(&PresetInputArgs {
            name: Some("Focus".to_string()),
            ..PresetInputArgs::default()
        }));
    }
}
