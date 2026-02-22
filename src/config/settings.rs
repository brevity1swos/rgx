use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_engine")]
    pub default_engine: String,
    #[serde(default)]
    pub theme: ThemeSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeSettings {
    #[serde(default)]
    pub catppuccin: bool,
}

fn default_engine() -> String {
    "rust".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_engine: default_engine(),
            theme: ThemeSettings::default(),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = config_path();
        if let Some(path) = path {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(settings) = toml::from_str(&content) {
                        return settings;
                    }
                }
            }
        }
        Self::default()
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("rgx").join("config.toml"))
}
