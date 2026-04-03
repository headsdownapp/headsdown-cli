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
        None => bail!("Not authenticated. Run `hd auth` first."),
    }
}
