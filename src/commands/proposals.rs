use anyhow::Result;

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const PROPOSALS_QUERY: &str = r#"
query Proposals($latest: Int, $verdict: VerdictDecision) {
    proposals(latest: $latest, verdict: $verdict) {
        id
        description
        estimatedFiles
        estimatedMinutes
        model
        framework
        verdict
        verdictReason
        insertedAt
    }
}
"#;

pub async fn list(
    api_url: &str,
    latest: Option<i32>,
    verdict: Option<String>,
    json: bool,
) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let verdict = verdict.map(|v| v.to_uppercase());
    let variables = serde_json::json!({ "latest": latest, "verdict": verdict });
    let data = client.execute(PROPOSALS_QUERY, Some(variables)).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data["proposals"])?);
        return Ok(());
    }

    let proposals = data["proposals"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No proposals found"))?;

    println!();
    println!("  {}", format::styled_bold("Recent proposals"));
    println!();

    if proposals.is_empty() {
        println!("  {}", format::styled_dimmed("No proposals found."));
        println!();
        return Ok(());
    }

    for proposal in proposals {
        let id = proposal["id"].as_str().unwrap_or("-");
        let desc = proposal["description"].as_str().unwrap_or("-");
        let decision = proposal["verdict"]
            .as_str()
            .unwrap_or("UNKNOWN")
            .to_uppercase();
        let reason = proposal["verdictReason"].as_str().unwrap_or("-");
        let files = proposal["estimatedFiles"].as_i64().unwrap_or_default();
        let minutes = proposal["estimatedMinutes"].as_i64().unwrap_or_default();

        println!(
            "  {} {}",
            format::styled_dimmed("•"),
            format::styled_bold(desc)
        );
        println!(
            "    {} {}  {} ~{} files / ~{} min",
            format::styled_dimmed("Verdict:"),
            format::color_verdict(&decision),
            format::styled_dimmed("Scope:"),
            files,
            minutes
        );
        println!("    {} {}", format::styled_dimmed("Reason:"), reason);
        println!(
            "    {} {}",
            format::styled_dimmed("ID:"),
            format::styled_dimmed(id)
        );
        println!();
    }

    Ok(())
}
