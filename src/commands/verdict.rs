use anyhow::Result;
use owo_colors::OwoColorize;

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const SUBMIT_PROPOSAL_MUTATION: &str = r#"
mutation SubmitProposal($input: ProposalInput!) {
    submitProposal(input: $input) {
        decision
        reason
        proposalId
        evaluatedAt
    }
}
"#;

pub async fn run(
    api_url: &str,
    description: &str,
    files: Option<i32>,
    minutes: Option<i32>,
    model: Option<String>,
) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let mut input = serde_json::json!({
        "description": description,
        "agentRef": "headsdown-cli",
        "sourceRef": format!("cli:{}", std::process::id()),
    });

    if let Some(f) = files {
        input["estimatedFiles"] = serde_json::json!(f);
    }
    if let Some(m) = minutes {
        input["estimatedMinutes"] = serde_json::json!(m);
    }
    if let Some(ref model_name) = model {
        input["model"] = serde_json::json!(model_name);
    }

    let variables = serde_json::json!({ "input": input });
    let data = client
        .execute(SUBMIT_PROPOSAL_MUTATION, Some(variables))
        .await?;

    let verdict = &data["submitProposal"];
    let decision = verdict["decision"]
        .as_str()
        .unwrap_or("UNKNOWN")
        .to_uppercase();
    let reason = verdict["reason"].as_str().unwrap_or("No reason provided");

    println!();
    println!(
        "  {} {}",
        "Verdict:".dimmed(),
        format::color_verdict(&decision)
    );
    println!();
    println!("  {} {}", "Reason:".dimmed(), reason);

    // Show the proposal details
    println!();
    println!("  {} {}", "Task:".dimmed(), description);
    if let Some(f) = files {
        println!("  {} ~{} files", "Scope:".dimmed(), f);
    }
    if let Some(m) = minutes {
        println!("  {} ~{} minutes", "Estimate:".dimmed(), m);
    }
    if let Some(ref model_name) = model {
        println!("  {} {}", "Model:".dimmed(), model_name);
    }

    println!();
    Ok(())
}
