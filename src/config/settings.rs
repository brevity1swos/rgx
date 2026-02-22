use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::engine::EngineKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_engine")]
    pub default_engine: String,
    #[serde(default)]
    pub case_insensitive: bool,
    #[serde(default)]
    pub multiline: bool,
    #[serde(default)]
    pub dotall: bool,
    #[serde(default = "default_true")]
    pub unicode: bool,
    #[serde(default)]
    pub extended: bool,
    #[serde(default)]
    pub show_whitespace: bool,
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

fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_engine: default_engine(),
            case_insensitive: false,
            multiline: false,
            dotall: false,
            unicode: default_true(),
            extended: false,
            show_whitespace: false,
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

    pub fn parse_engine(&self) -> EngineKind {
        match self.default_engine.as_str() {
            "fancy" => EngineKind::FancyRegex,
            #[cfg(feature = "pcre2-engine")]
            "pcre2" => EngineKind::Pcre2,
            _ => EngineKind::RustRegex,
        }
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("rgx").join("config.toml"))
}
