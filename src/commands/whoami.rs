use anyhow::Result;

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const PROFILE_QUERY: &str = r#"
query {
    profile {
        id
        name
        email
        location
    }
}
"#;

pub async fn run(api_url: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let data = client.execute(PROFILE_QUERY, None).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data["profile"])?);
        return Ok(());
    }

    let profile = &data["profile"];
    let name = profile["name"].as_str().unwrap_or("Unknown");
    let email = profile["email"].as_str().unwrap_or("Unknown");
    let location = profile["location"].as_str();

    println!();
    println!(
        "  {} {}",
        format::styled_green_bold("✓"),
        format::styled_bold(name)
    );
    println!("  {} {}", format::styled_dimmed("Email:"), email);
    if let Some(loc) = location {
        println!("  {} {}", format::styled_dimmed("Location:"), loc);
    }
    println!(
        "  {} {}",
        format::styled_dimmed("API:"),
        format::styled_dimmed(api_url)
    );
    println!();

    Ok(())
}
