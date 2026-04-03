use anyhow::{bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use serde::Deserialize;
use std::time::Duration;

use crate::auth;

#[derive(Deserialize)]
struct DeviceAuthResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[allow(dead_code)]
    verification_uri_complete: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    #[allow(dead_code)]
    error_description: Option<String>,
}

pub async fn run(api_url: &str) -> Result<()> {
    // Check if already authenticated
    if let Some(token) = auth::load_token()? {
        let client = crate::client::GraphQLClient::new(api_url, &token);
        let query = r#"query { profile { name email } }"#;
        match client.execute(query, None).await {
            Ok(data) => {
                let profile = &data["profile"];
                let name = profile["name"].as_str().unwrap_or("Unknown");
                let email = profile["email"].as_str().unwrap_or("Unknown");

                println!();
                println!(
                    "  {} Authenticated as {} ({})",
                    "✓".green().bold(),
                    name.bold(),
                    email.dimmed()
                );
                println!();
                print!("  Re-authenticate? [y/N] ");

                use std::io::{self, Write};
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if !input.trim().eq_ignore_ascii_case("y") {
                    return Ok(());
                }
                println!();
            }
            Err(_) => {
                println!(
                    "  {} Stored credentials are invalid. Starting fresh authentication...",
                    "!".yellow().bold()
                );
                println!();
            }
        }
    }

    // Step 1: Request device authorization
    let http = reqwest::Client::new();
    let device_url = format!("{}/oauth/device", api_url);

    let resp = http
        .post(&device_url)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "label": "HeadsDown CLI" }))
        .send()
        .await
        .context("Failed to connect to HeadsDown API")?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("Device authorization failed: {}", body);
    }

    let device_auth: DeviceAuthResponse = resp.json().await?;

    // Step 2: Display the user code and verification URL
    println!();
    println!("  {} Open this URL in your browser:", "→".cyan().bold());
    println!();
    println!("    {}", device_auth.verification_uri.cyan().underline());
    println!();
    println!("  {} Enter this code when prompted:", "→".cyan().bold());
    println!();
    println!("    {}", device_auth.user_code.bold().yellow());
    println!();

    // Step 3: Poll for approval with a spinner
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("  {spinner:.cyan} Waiting for approval...")
            .unwrap(),
    );
    spinner.enable_steady_tick(Duration::from_millis(100));

    let token_url = format!("{}/oauth/token", api_url);
    let interval = Duration::from_secs(device_auth.interval);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(device_auth.expires_in);
    let grant_type = "urn:ietf:params:oauth:grant-type:device_code";

    loop {
        tokio::time::sleep(interval).await;

        if tokio::time::Instant::now() > deadline {
            spinner.finish_and_clear();
            bail!("Authorization timed out. Please run `hd auth` again.");
        }

        let resp = http
            .post(&token_url)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "grant_type": grant_type,
                "device_code": device_auth.device_code,
            }))
            .send()
            .await?;

        let token_resp: TokenResponse = resp.json().await?;

        if let Some(access_token) = token_resp.access_token {
            spinner.finish_and_clear();

            // Store the token
            auth::store_token(&access_token)?;

            // Fetch profile to show who logged in
            let client = crate::client::GraphQLClient::new(api_url, &access_token);
            let query = r#"query { profile { name email } }"#;
            let name = match client.execute(query, None).await {
                Ok(data) => {
                    let profile = &data["profile"];
                    format!(
                        "{} ({})",
                        profile["name"].as_str().unwrap_or("Unknown"),
                        profile["email"].as_str().unwrap_or("Unknown")
                    )
                }
                Err(_) => "Unknown".to_string(),
            };

            println!("  {} Authenticated as {}", "✓".green().bold(), name.bold());
            println!();
            return Ok(());
        }

        match token_resp.error.as_deref() {
            Some("authorization_pending") => continue,
            Some("slow_down") => {
                // Back off: add 5 seconds to the interval per RFC 8628
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
            Some("access_denied") => {
                spinner.finish_and_clear();
                bail!("Authorization was denied by the user.");
            }
            Some("expired_token") => {
                spinner.finish_and_clear();
                bail!("Device code expired. Please run `hd auth` again.");
            }
            Some(err) => {
                spinner.finish_and_clear();
                bail!("Authentication error: {}", err);
            }
            None => continue,
        }
    }
}
