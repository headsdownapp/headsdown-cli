use anyhow::{bail, Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
struct JsonCredentials {
    #[serde(rename = "apiKey")]
    api_key: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    label: Option<String>,
}

/// Returns the path to the config directory, respecting XDG_CONFIG_HOME.
fn config_dir() -> Result<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        Ok(PathBuf::from(xdg).join("headsdown"))
    } else if let Some(proj) = ProjectDirs::from("app", "headsdown", "headsdown") {
        Ok(proj.config_dir().to_path_buf())
    } else {
        bail!("Could not determine config directory");
    }
}

fn legacy_credentials_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("credentials"))
}

fn json_credentials_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("credentials.json"))
}

/// Load the stored API key, if any. Supports both legacy plain-token credentials and modern credentials.json.
pub fn load_token() -> Result<Option<String>> {
    let json_path = json_credentials_path()?;
    if json_path.exists() {
        let contents = fs::read_to_string(&json_path)
            .with_context(|| format!("Failed to read {}", json_path.display()))?;
        if let Ok(parsed) = serde_json::from_str::<JsonCredentials>(&contents) {
            let token = parsed.api_key.trim().to_string();
            if !token.is_empty() {
                return Ok(Some(token));
            }
        }
    }

    let legacy_path = legacy_credentials_path()?;
    if legacy_path.exists() {
        let contents = fs::read_to_string(&legacy_path)
            .with_context(|| format!("Failed to read {}", legacy_path.display()))?;
        let token = contents.trim().to_string();
        if token.is_empty() {
            Ok(None)
        } else {
            Ok(Some(token))
        }
    } else {
        Ok(None)
    }
}

/// Store the API key to credentials.json (modern format) and legacy credentials for backwards compatibility.
pub fn store_token(token: &str) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create directory {}", dir.display()))?;

    let trimmed = token.trim();

    let json_path = json_credentials_path()?;
    let json_payload = JsonCredentials {
        api_key: trimmed.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        label: Some("HeadsDown CLI".to_string()),
    };
    write_secure(
        &json_path,
        &(serde_json::to_string_pretty(&json_payload)? + "\n"),
    )?;

    let legacy_path = legacy_credentials_path()?;
    write_secure(&legacy_path, trimmed)?;

    Ok(())
}

fn write_secure(path: &PathBuf, contents: &str) -> Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        file.write_all(contents.as_bytes())?;
    }

    #[cfg(not(unix))]
    {
        fs::write(path, contents).with_context(|| format!("Failed to write {}", path.display()))?;
    }

    Ok(())
}

/// Require a stored token, returning an error with a helpful message if not found.
pub fn require_token() -> Result<String> {
    match load_token()? {
        Some(token) => Ok(token),
        None => bail!("Not authenticated. Run `hd auth` first"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn with_temp_config<F: FnOnce()>(f: F) {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        f();
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    #[serial]
    fn load_token_returns_none_when_no_file() {
        with_temp_config(|| {
            assert_eq!(load_token().unwrap(), None);
        });
    }

    #[test]
    #[serial]
    fn store_then_load_round_trips() {
        with_temp_config(|| {
            store_token("hd_abc123").unwrap();
            assert_eq!(load_token().unwrap(), Some("hd_abc123".to_string()));
        });
    }

    #[test]
    #[serial]
    fn load_reads_json_credentials_when_present() {
        with_temp_config(|| {
            let path = json_credentials_path().unwrap();
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(
                &path,
                r#"{"apiKey":"hd_from_json","createdAt":"2026-01-01T00:00:00Z"}"#,
            )
            .unwrap();
            assert_eq!(load_token().unwrap(), Some("hd_from_json".to_string()));
        });
    }

    #[test]
    #[serial]
    fn require_token_errors_when_no_token() {
        with_temp_config(|| {
            let err = require_token().unwrap_err();
            assert!(err.to_string().contains("Not authenticated"));
        });
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn store_token_sets_600_permissions() {
        use std::os::unix::fs::PermissionsExt;
        with_temp_config(|| {
            store_token("hd_secret").unwrap();
            let metadata = fs::metadata(json_credentials_path().unwrap()).unwrap();
            let mode = metadata.permissions().mode() & 0o777;
            assert_eq!(mode, 0o600, "Credentials file should be owner-only (0600)");
        });
    }
}
