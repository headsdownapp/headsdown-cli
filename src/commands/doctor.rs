use anyhow::Result;

use crate::auth;
use crate::config;
use crate::format;

pub async fn run(api_url: &str, json: bool) -> Result<()> {
    let mut checks: Vec<(&str, bool, String)> = Vec::new();

    // Check 1: CLI version
    let version = env!("CARGO_PKG_VERSION");
    checks.push(("CLI version", true, version.to_string()));

    // Check 2: Config directory
    let config_ok = config::config_dir().is_ok();
    let config_detail = match config::config_dir() {
        Ok(dir) => format!("{}", dir.display()),
        Err(e) => format!("Error: {}", e),
    };
    checks.push(("Config directory", config_ok, config_detail));

    // Check 3: Credentials
    let creds = auth::load_token();
    let creds_ok = matches!(&creds, Ok(Some(_)));
    let creds_detail = match &creds {
        Ok(Some(token)) => {
            if token.starts_with("hd_") {
                format!(
                    "{}...{}",
                    &token[..6],
                    &token[token.len().saturating_sub(4)..]
                )
            } else {
                "Present (unknown format)".to_string()
            }
        }
        Ok(None) => "Not found. Run `hd auth`".to_string(),
        Err(e) => format!("Error: {}", e),
    };
    checks.push(("Credentials", creds_ok, creds_detail));

    // Check 4: API connectivity
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();
    let api_check = http.get(format!("{}/healthz", api_url)).send().await;
    let (api_ok, api_detail) = match api_check {
        Ok(resp) => {
            let status = resp.status();
            (
                status.is_success(),
                format!("{} (HTTP {})", api_url, status),
            )
        }
        Err(e) => (false, format!("Cannot reach {}: {}", api_url, e)),
    };
    checks.push(("API connectivity", api_ok, api_detail));

    // Check 5: Authentication validity
    let auth_ok;
    let auth_detail;
    if let Ok(Some(token)) = &creds {
        let client = crate::client::GraphQLClient::new(api_url, token);
        match client
            .execute(r#"query { profile { name email } }"#, None)
            .await
        {
            Ok(data) => {
                let name = data["profile"]["name"].as_str().unwrap_or("Unknown");
                let email = data["profile"]["email"].as_str().unwrap_or("Unknown");
                auth_ok = true;
                auth_detail = format!("{} ({})", name, email);
            }
            Err(e) => {
                auth_ok = false;
                auth_detail = format!("Token invalid: {}", e);
            }
        }
    } else {
        auth_ok = false;
        auth_detail = "Skipped (no credentials)".to_string();
    };
    checks.push(("Authentication", auth_ok, auth_detail));

    // Check 6: Config file
    let cfg_result = config::load();
    let (cfg_ok, cfg_detail) = match cfg_result {
        Ok(_) => (true, "Valid".to_string()),
        Err(e) => (false, format!("Error: {}", e)),
    };
    checks.push(("Config file", cfg_ok, cfg_detail));

    // Check 7: OS/Arch
    checks.push((
        "Platform",
        true,
        format!("{}/{}", std::env::consts::OS, std::env::consts::ARCH),
    ));

    if json {
        let json_checks: Vec<serde_json::Value> = checks
            .iter()
            .map(|(name, ok, detail)| {
                serde_json::json!({
                    "check": name,
                    "ok": ok,
                    "detail": detail,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_checks)?);
        return Ok(());
    }

    // Human output
    println!();
    println!("  {}", format::styled_bold("HeadsDown CLI Health Check"));
    println!();

    let mut all_ok = true;
    for (name, ok, detail) in &checks {
        let icon = if *ok {
            format::styled_green_bold("✓")
        } else {
            all_ok = false;
            format::styled_yellow_bold("✗")
        };
        println!("  {} {} {}", icon, format::styled_dimmed(name), detail);
    }

    println!();
    if all_ok {
        println!("  {}", format::styled_green_bold("All checks passed"));
    } else {
        println!(
            "  {}",
            format::styled_yellow_bold("Some checks failed. See details above")
        );
    }
    println!();

    Ok(())
}
