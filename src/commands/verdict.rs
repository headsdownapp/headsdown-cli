use anyhow::Result;

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const SUBMIT_PROPOSAL_MUTATION: &str = r#"
mutation SubmitProposal($input: ProposalInput!) {
    submitProposal(input: $input) {
        decision
        policy
        policyStatus
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
    json: bool,
) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let mut input = serde_json::json!({
        "description": description,
        "agentRef": "headsdown-cli",
        "sourceRef": "cli",
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

    if json {
        println!("{}", serde_json::to_string_pretty(&data["submitProposal"])?);
        return Ok(());
    }

    let verdict = &data["submitProposal"];
    let decision = verdict["decision"]
        .as_str()
        .unwrap_or("UNKNOWN")
        .to_uppercase();
    let policy = verdict["policy"].as_str().unwrap_or("UNKNOWN");
    let policy_status = verdict["policyStatus"].as_str().unwrap_or("UNKNOWN");
    let reason = verdict["reason"].as_str().unwrap_or("No reason provided");

    println!();
    println!(
        "  {} {}",
        format::styled_dimmed("Verdict:"),
        format::color_verdict(&decision)
    );
    println!();
    println!("  {} {}", format::styled_dimmed("Policy:"), policy);
    println!("  {} {}", format::styled_dimmed("State:"), policy_status);
    println!("  {} {}", format::styled_dimmed("Reason:"), reason);

    // Show the proposal details
    println!();
    println!("  {} {}", format::styled_dimmed("Task:"), description);
    if let Some(f) = files {
        println!("  {} ~{} files", format::styled_dimmed("Scope:"), f);
    }
    if let Some(m) = minutes {
        println!("  {} ~{} minutes", format::styled_dimmed("Estimate:"), m);
    }
    if let Some(ref model_name) = model {
        println!("  {} {}", format::styled_dimmed("Model:"), model_name);
    }

    println!();
    Ok(())
}
