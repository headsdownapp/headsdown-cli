use anyhow::{bail, Result};

use crate::auth;
use crate::client::GraphQLClient;
use crate::config;
use crate::format;

const REPORT_OUTCOME_MUTATION: &str = r#"
mutation ReportOutcome($input: OutcomeInput!) {
    reportOutcome(input: $input) {
        id
        outcome
        actualDurationMinutes
        filesModified
        linesChanged
        turnCount
        dataQualityScore
        insertedAt
    }
}
"#;

pub async fn run(
    api_url: &str,
    proposal_id: &str,
    outcome: &str,
    duration: Option<i32>,
    files: Option<i32>,
    lines: Option<i32>,
    turns: Option<i32>,
    error_category: Option<String>,
    tests_passed: Option<bool>,
    json: bool,
) -> Result<()> {
    // Check calibration is enabled
    let cfg = config::load()?;
    if !cfg.calibration.enabled {
        if json {
            let response = serde_json::json!({
                "error": "calibration_disabled",
                "message": "Calibration is disabled. Enable with: hd calibration on"
            });
            println!("{}", serde_json::to_string_pretty(&response)?);
        } else {
            println!();
            println!(
                "  {} Calibration is disabled. Enable with: hd calibration on",
                format::styled_dimmed("ℹ")
            );
            println!();
        }
        return Ok(());
    }

    let valid_outcomes = [
        "completed",
        "failed",
        "partially_completed",
        "cancelled",
        "timed_out",
    ];
    if !valid_outcomes.contains(&outcome) {
        bail!(
            "Invalid outcome '{}'. Must be one of: {}",
            outcome,
            valid_outcomes.join(", ")
        );
    }

    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let mut input = serde_json::json!({
        "proposalId": proposal_id,
        "outcome": outcome.to_uppercase(),
    });

    if let Some(d) = duration {
        input["actualDurationMinutes"] = serde_json::json!(d);
    }
    if let Some(f) = files {
        input["filesModified"] = serde_json::json!(f);
    }
    if let Some(l) = lines {
        input["linesChanged"] = serde_json::json!(l);
    }
    if let Some(t) = turns {
        input["turnCount"] = serde_json::json!(t);
    }
    if let Some(ref cat) = error_category {
        input["errorCategory"] = serde_json::json!(cat);
    }
    if let Some(tp) = tests_passed {
        input["testsPassed"] = serde_json::json!(tp);
    }

    let variables = serde_json::json!({ "input": input });
    let data = client
        .execute(REPORT_OUTCOME_MUTATION, Some(variables))
        .await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data["reportOutcome"])?);
        return Ok(());
    }

    let result = &data["reportOutcome"];
    let score = result["dataQualityScore"]
        .as_f64()
        .map(|s| format!("{:.0}%", s * 100.0))
        .unwrap_or_else(|| "N/A".to_string());

    println!();
    println!("  {} Outcome recorded", format::styled_green_bold("✓"));
    println!();
    println!("  {} {}", format::styled_dimmed("Outcome:"), outcome);
    println!("  {} {}", format::styled_dimmed("Quality:"), score);
    if let Some(d) = duration {
        println!("  {} {} min", format::styled_dimmed("Duration:"), d);
    }
    if let Some(f) = files {
        println!("  {} {} files", format::styled_dimmed("Files:"), f);
    }
    println!();
    Ok(())
}
