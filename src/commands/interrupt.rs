use anyhow::Result;

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const EVALUATE_INTERRUPT_QUERY: &str = r#"
query EvaluateInterrupt($handle: String) {
    evaluateInterrupt(handle: $handle) {
        allowed
        reason
        autoResponse
    }
}
"#;

pub async fn evaluate(api_url: &str, handle: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let data = client
        .execute(
            EVALUATE_INTERRUPT_QUERY,
            Some(serde_json::json!({ "handle": handle })),
        )
        .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data["evaluateInterrupt"])?
        );
        return Ok(());
    }

    let result = &data["evaluateInterrupt"];
    let allowed = result["allowed"].as_bool().unwrap_or(false);

    println!();
    println!("  {} {}", format::styled_dimmed("Handle:"), handle);
    println!(
        "  {} {}",
        format::styled_dimmed("Allowed:"),
        if allowed {
            format::styled_green_bold("yes")
        } else {
            format::styled_yellow_bold("no")
        }
    );
    if let Some(reason) = result["reason"].as_str() {
        println!("  {} {}", format::styled_dimmed("Reason:"), reason);
    }
    if let Some(response) = result["autoResponse"].as_str() {
        if !response.is_empty() {
            println!("  {} {}", format::styled_dimmed("Auto-response:"), response);
        }
    }
    println!();

    Ok(())
}
