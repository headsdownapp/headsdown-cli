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
    /// Calibration settings (improves verdict accuracy over time)
    pub calibration: CalibrationConfig,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CalibrationConfig {
    /// Whether calibration reporting is enabled (improves verdict accuracy)
    pub enabled: bool,
}

impl Default for CalibrationConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
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
    fn load_returns_default_when_no_file_exists() {
        with_temp_config(|| {
            let cfg = load().unwrap();
            assert!(cfg.default_duration.is_none());
            assert!(cfg.default_model.is_none());
            assert!(cfg.api_url.is_none());
            assert!(!cfg.telemetry.enabled);
            assert!(cfg.calibration.enabled); // calibration defaults to enabled
            assert!(cfg.aliases.is_empty());
        });
    }

    #[test]
    #[serial]
    fn save_then_load_round_trips() {
        with_temp_config(|| {
            let mut cfg = Config::default();
            cfg.default_duration = Some(120);
            cfg.default_model = Some("gpt-4".to_string());
            cfg.api_url = Some("https://custom.example.com".to_string());
            cfg.telemetry.enabled = true;
            cfg.calibration.enabled = false; // Test non-default value
            cfg.aliases
                .insert("focus".to_string(), "busy 2h".to_string());
            cfg.aliases.insert("brb".to_string(), "offline".to_string());

            save(&cfg).unwrap();
            let loaded = load().unwrap();

            assert_eq!(loaded.default_duration, Some(120));
            assert_eq!(loaded.default_model.as_deref(), Some("gpt-4"));
            assert_eq!(
                loaded.api_url.as_deref(),
                Some("https://custom.example.com")
            );
            assert!(loaded.telemetry.enabled);
            assert!(!loaded.calibration.enabled);
            assert_eq!(
                loaded.aliases.get("focus").map(|s| s.as_str()),
                Some("busy 2h")
            );
            assert_eq!(
                loaded.aliases.get("brb").map(|s| s.as_str()),
                Some("offline")
            );
        });
    }

    #[test]
    #[serial]
    fn update_modifies_existing_config() {
        with_temp_config(|| {
            let mut cfg = Config::default();
            cfg.default_duration = Some(60);
            cfg.default_model = Some("claude".to_string());
            save(&cfg).unwrap();

            update(|c| c.default_duration = Some(120)).unwrap();

            let loaded = load().unwrap();
            assert_eq!(loaded.default_duration, Some(120));
            // Other fields preserved
            assert_eq!(loaded.default_model.as_deref(), Some("claude"));
        });
    }

    #[test]
    #[serial]
    fn update_creates_file_if_missing() {
        with_temp_config(|| {
            update(|c| c.telemetry.enabled = true).unwrap();
            let loaded = load().unwrap();
            assert!(loaded.telemetry.enabled);
        });
    }

    #[test]
    #[serial]
    fn load_returns_error_for_malformed_toml() {
        with_temp_config(|| {
            let path = config_path().unwrap();
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, "{{{{ not valid toml").unwrap();
            assert!(load().is_err());
        });
    }

    #[test]
    #[serial]
    fn unknown_toml_keys_are_ignored() {
        with_temp_config(|| {
            let path = config_path().unwrap();
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, "unknown_key = \"bar\"\ndefault_duration = 90\n").unwrap();
            let cfg = load().unwrap();
            assert_eq!(cfg.default_duration, Some(90));
        });
    }

    #[test]
    #[serial]
    fn aliases_round_trip_through_toml() {
        with_temp_config(|| {
            let mut cfg = Config::default();
            cfg.aliases
                .insert("focus".to_string(), "busy 2h".to_string());
            cfg.aliases.insert("brb".to_string(), "offline".to_string());
            save(&cfg).unwrap();

            let loaded = load().unwrap();
            assert_eq!(loaded.aliases.len(), 2);
            assert_eq!(loaded.aliases["focus"], "busy 2h");
            assert_eq!(loaded.aliases["brb"], "offline");
        });
    }

    #[test]
    #[serial]
    fn calibration_defaults_to_enabled() {
        with_temp_config(|| {
            let cfg = Config::default();
            assert!(cfg.calibration.enabled);
        });
    }

    #[test]
    #[serial]
    fn calibration_can_be_toggled() {
        with_temp_config(|| {
            // Start with default (enabled)
            let cfg = Config::default();
            assert!(cfg.calibration.enabled);
            save(&cfg).unwrap();

            // Disable it
            update(|c| c.calibration.enabled = false).unwrap();
            let loaded = load().unwrap();
            assert!(!loaded.calibration.enabled);

            // Enable it again
            update(|c| c.calibration.enabled = true).unwrap();
            let loaded = load().unwrap();
            assert!(loaded.calibration.enabled);
        });
    }
}
