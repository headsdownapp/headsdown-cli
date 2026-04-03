use anyhow::{bail, Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

/// Returns the path to the credentials file, respecting XDG_CONFIG_HOME.
fn credentials_path() -> Result<PathBuf> {
    let dir = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg).join("headsdown")
    } else if let Some(proj) = ProjectDirs::from("app", "headsdown", "headsdown") {
        proj.config_dir().to_path_buf()
    } else {
        bail!("Could not determine config directory");
    };

    Ok(dir.join("credentials"))
}

/// Load the stored API key, if any.
pub fn load_token() -> Result<Option<String>> {
    let path = credentials_path()?;
    if path.exists() {
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
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

/// Store the API key to the credentials file.
pub fn store_token(token: &str) -> Result<()> {
    let path = credentials_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    // Write with restrictive permissions (owner read/write only)
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        file.write_all(token.as_bytes())?;
    }

    #[cfg(not(unix))]
    {
        fs::write(&path, token).with_context(|| format!("Failed to write {}", path.display()))?;
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
    fn store_creates_parent_directories() {
        with_temp_config(|| {
            // Temp dir has no headsdown/ subdir yet
            store_token("hd_test").unwrap();
            assert!(credentials_path().unwrap().exists());
        });
    }

    #[test]
    #[serial]
    fn load_returns_none_for_empty_file() {
        with_temp_config(|| {
            let path = credentials_path().unwrap();
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, "").unwrap();
            assert_eq!(load_token().unwrap(), None);
        });
    }

    #[test]
    #[serial]
    fn load_returns_none_for_whitespace_only_file() {
        with_temp_config(|| {
            let path = credentials_path().unwrap();
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, "  \n  ").unwrap();
            assert_eq!(load_token().unwrap(), None);
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

    #[test]
    #[serial]
    fn require_token_returns_token_when_present() {
        with_temp_config(|| {
            store_token("hd_xyz").unwrap();
            assert_eq!(require_token().unwrap(), "hd_xyz");
        });
    }

    #[test]
    #[serial]
    fn store_empty_string_then_load_returns_none() {
        with_temp_config(|| {
            store_token("").unwrap();
            assert_eq!(load_token().unwrap(), None);
        });
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn store_token_sets_600_permissions() {
        use std::os::unix::fs::PermissionsExt;
        with_temp_config(|| {
            store_token("hd_secret").unwrap();
            let path = credentials_path().unwrap();
            let metadata = fs::metadata(&path).unwrap();
            let mode = metadata.permissions().mode() & 0o777;
            assert_eq!(mode, 0o600, "Credentials file should be owner-only (0600)");
        });
    }
}
