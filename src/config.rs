use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// User configuration stored at ~/.config/headsdown/config.toml
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Default mode duration in minutes (e.g. 120 for 2h)
    pub default_duration: Option<i64>,
    /// Default AI model for verdict command
    pub default_model: Option<String>,
    /// API base URL override
    pub api_url: Option<String>,
    /// Telemetry settings
    pub telemetry: TelemetryConfig,
    /// User-defined command aliases
    #[serde(default)]
    pub aliases: HashMap<String, String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TelemetryConfig {
    /// Whether anonymous usage telemetry is enabled
    pub enabled: bool,
}

/// Returns the config directory path, respecting XDG_CONFIG_HOME.
pub fn config_dir() -> Result<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        Ok(PathBuf::from(xdg).join("headsdown"))
    } else if let Some(proj) = ProjectDirs::from("app", "headsdown", "headsdown") {
        Ok(proj.config_dir().to_path_buf())
    } else {
        anyhow::bail!("Could not determine config directory")
    }
}

/// Returns the path to the config file.
fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

/// Load config from disk. Returns default config if file doesn't exist.
pub fn load() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    toml::from_str(&contents).with_context(|| format!("Failed to parse {}", path.display()))
}

/// Save config to disk.
pub fn save(config: &Config) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = toml::to_string_pretty(config).context("Failed to serialize config")?;
    fs::write(&path, contents).with_context(|| format!("Failed to write {}", path.display()))
}

/// Get a mutable reference to the config, load it, apply changes, and save.
pub fn update(f: impl FnOnce(&mut Config)) -> Result<()> {
    let mut config = load()?;
    f(&mut config);
    save(&config)
}
