use anyhow::{Context, Result, bail};
use directories::{ProjectDirs, UserDirs};
use serde::Deserialize;
use std::{env, fs, path::PathBuf};

pub const APP_NAME: &str = "boat";
pub const CONFIG_VAR: &str = "BOAT_CONFIG";
pub const DEFAULT_CONFIG_FILE: &str = "config.toml";
pub const TUI_CONFIG_FILE: &str = "tui.toml";

/// Subset of the boat-cli configuration that the TUI needs.
/// Unknown fields are ignored so the TUI keeps working when boat-cli adds new settings.
#[derive(Debug, Deserialize)]
pub struct BoatConfig {
    pub database_path: PathBuf,
}

/// TUI-specific settings, read from `tui.toml` next to the boat config file.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct TuiConfig {
    /// Directory where per-activity notes are stored (`<notes_dir>/<activity id>/note.md`)
    pub notes_dir: PathBuf,

    /// Base URL used to open `jira:<ISSUE>` tags, e.g. `https://acme.atlassian.net`
    pub jira_base_url: Option<String>,

    /// Meeting presets shown by the meeting action
    pub meetings: Vec<MeetingPreset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MeetingPreset {
    /// Label displayed in the picker
    pub label: String,

    /// Tag identifying the meeting activity, e.g. `meeting:daily`
    pub tag: String,

    /// Create a brand new activity instead of resuming the latest one with this tag
    #[serde(default)]
    pub create: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        let notes_dir = UserDirs::new()
            .and_then(|dirs| dirs.document_dir().map(|d| d.join("boat-acts")))
            .unwrap_or_else(|| PathBuf::from("boat-acts"));

        Self {
            notes_dir,
            jira_base_url: None,
            meetings: vec![
                MeetingPreset {
                    label: "Daily meeting".to_string(),
                    tag: "meeting:daily".to_string(),
                    create: false,
                },
                MeetingPreset {
                    label: "Weekly meeting".to_string(),
                    tag: "meeting:weekly".to_string(),
                    create: false,
                },
                MeetingPreset {
                    label: "Miscellaneous meeting".to_string(),
                    tag: "meeting:misc".to_string(),
                    create: true,
                },
            ],
        }
    }
}

pub struct Config {
    pub config_file: PathBuf,
    pub boat: BoatConfig,
    pub tui: TuiConfig,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_file = get_config_file_path()?;
        if !config_file.exists() {
            bail!(
                "boat config not found at {}; run `boat init` first",
                config_file.display()
            );
        }

        let content = fs::read_to_string(&config_file)
            .with_context(|| format!("failed to read {}", config_file.display()))?;
        let boat: BoatConfig = toml::from_str(&content)
            .with_context(|| format!("failed to parse {}", config_file.display()))?;

        let tui_file = config_file.with_file_name(TUI_CONFIG_FILE);
        let tui = if tui_file.exists() {
            let content = fs::read_to_string(&tui_file)
                .with_context(|| format!("failed to read {}", tui_file.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("failed to parse {}", tui_file.display()))?
        } else {
            TuiConfig::default()
        };

        Ok(Self {
            config_file,
            boat,
            tui,
        })
    }
}

/// Same resolution rules as boat-cli: `$BOAT_CONFIG`, then the platform config dir.
fn get_config_file_path() -> Result<PathBuf> {
    if let Ok(config_var) = env::var(CONFIG_VAR) {
        return Ok(PathBuf::from(config_var));
    }

    if let Some(proj_dirs) = ProjectDirs::from("", "", APP_NAME) {
        return Ok(proj_dirs.config_dir().join(DEFAULT_CONFIG_FILE));
    }

    bail!("could not get config directory")
}
