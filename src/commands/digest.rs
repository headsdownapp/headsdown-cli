use anyhow::Result;

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const DIGEST_QUERY: &str = r#"
query DigestSummaries($latest: Int) {
    digestSummaries(latest: $latest) {
        id
        action
        actorLabel
        actorRef
        channelRef
        sourceType
        entryCount
        firstEventAt
        lastEventAt
        events {
            description
            insertedAt
        }
    }
}
"#;

const DISMISS_DIGEST_MUTATION: &str = r#"
mutation DismissDigestEntry($id: ID!) {
    dismissDigestEntry(id: $id) {
        id
        action
        actorLabel
        entryCount
        sourceType
    }
}
"#;

pub async fn list(api_url: &str, latest: Option<i32>, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let variables = serde_json::json!({ "latest": latest });
    let data = client.execute(DIGEST_QUERY, Some(variables)).await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data["digestSummaries"])?
        );
        return Ok(());
    }

    let entries = data["digestSummaries"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No digest summaries found"))?;

    println!();
    println!("  {}", format::styled_bold("Digest Summaries"));
    println!();

    if entries.is_empty() {
        println!("  {}", format::styled_dimmed("No digest entries."));
        println!();
        return Ok(());
    }

    for entry in entries {
        let id = entry["id"].as_str().unwrap_or("-");
        let action = entry["action"].as_str().unwrap_or("-");
        let actor = entry["actorLabel"]
            .as_str()
            .or_else(|| entry["actorRef"].as_str())
            .unwrap_or("Unknown");
        let count = entry["entryCount"].as_i64().unwrap_or_default();
        let source = entry["sourceType"].as_str().unwrap_or("-");

        println!(
            "  {} {} ({})",
            format::styled_dimmed("•"),
            format::styled_bold(actor),
            action.to_lowercase()
        );
        println!(
            "    {} {}  {} {}",
            format::styled_dimmed("Entries:"),
            count,
            format::styled_dimmed("Source:"),
            source.to_lowercase()
        );
        println!(
            "    {} {}",
            format::styled_dimmed("ID:"),
            format::styled_dimmed(id)
        );

        if let Some(events) = entry["events"].as_array() {
            for event in events.iter().take(2) {
                if let Some(description) = event["description"].as_str() {
                    println!(
                        "    {} {}",
                        format::styled_dimmed("-"),
                        format::styled_dimmed(description)
                    );
                }
            }
        }

        println!();
    }

    Ok(())
}

pub async fn dismiss(api_url: &str, id: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let data = client
        .execute(
            DISMISS_DIGEST_MUTATION,
            Some(serde_json::json!({ "id": id })),
        )
        .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data["dismissDigestEntry"])?
        );
        return Ok(());
    }

    let entry = &data["dismissDigestEntry"];
    println!();
    println!(
        "  {} Dismissed digest entry",
        format::styled_green_bold("✓")
    );
    println!(
        "  {} {}",
        format::styled_dimmed("ID:"),
        format::styled_dimmed(entry["id"].as_str().unwrap_or(id))
    );
    println!();

    Ok(())
}
