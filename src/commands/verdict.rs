use anyhow::Result;
use serde::{Deserialize, Serialize};

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
        wrapUpGuidance {
            active
            deadlineAt
            remainingMinutes
            profile
            source
            reason
            hints
            thresholdMinutes
            selectedMode
        }
    }
}
"#;

#[derive(Deserialize)]
struct SubmitProposalResponse {
    #[serde(rename = "submitProposal")]
    submit_proposal: Verdict,
}

#[derive(Deserialize, Serialize)]
struct Verdict {
    decision: String,
    reason: String,
    #[serde(rename = "proposalId")]
    proposal_id: String,
    #[serde(rename = "wrapUpGuidance")]
    wrap_up_guidance: Option<WrapUpGuidance>,
}

#[derive(Deserialize, Serialize)]
struct WrapUpGuidance {
    #[serde(rename = "selectedMode")]
    selected_mode: Option<String>,
    #[serde(rename = "remainingMinutes")]
    remaining_minutes: Option<i64>,
}

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
    let data: SubmitProposalResponse = client
        .execute_typed(SUBMIT_PROPOSAL_MUTATION, Some(variables))
        .await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data.submit_proposal)?);
        return Ok(());
    }

    let verdict = data.submit_proposal;
    let decision = verdict.decision.to_uppercase();
    let reason = verdict.reason;

    println!();
    println!(
        "  {} {}",
        format::styled_dimmed("Verdict:"),
        format::color_verdict(&decision)
    );
    println!();
    println!("  {} {}", format::styled_dimmed("Reason:"), reason);

    if let Some(wrap_up_guidance) = verdict.wrap_up_guidance {
        if let Some(mode) = wrap_up_guidance.selected_mode {
            println!("  {} {}", format::styled_dimmed("Delivery mode:"), mode);
        }
        if let Some(minutes) = wrap_up_guidance.remaining_minutes {
            println!(
                "  {} {} min",
                format::styled_dimmed("Attention window:"),
                minutes
            );
        }
    }

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
