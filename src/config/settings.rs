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
    pub rounded_borders: bool,
    #[serde(default)]
    pub vim_mode: bool,
    #[serde(default = "default_debug_max_steps")]
    pub debug_max_steps: usize,
}

fn default_engine() -> String {
    "rust".to_string()
}

const fn default_true() -> bool {
    true
}

const fn default_debug_max_steps() -> usize {
    10_000
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
            rounded_borders: false,
            vim_mode: false,
            debug_max_steps: default_debug_max_steps(),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        Self::load_with_warning().0
    }

    /// Load the config file, falling back to defaults. The warning explains a
    /// config that exists but could not be used, so it is never ignored silently.
    pub fn load_with_warning() -> (Self, Option<String>) {
        let Some(path) = config_path().filter(|p| p.exists()) else {
            return (Self::default(), None);
        };
        let parsed = std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|content| toml::from_str::<Self>(&content).map_err(|e| e.to_string()));
        match parsed {
            Ok(settings) => {
                let warning = EngineKind::from_name(&settings.default_engine)
                    .err()
                    .map(|e| format!("{}: default_engine: {e}; using rust", path.display()));
                (settings, warning)
            }
            Err(e) => (
                Self::default(),
                Some(format!("{}: {e}; using defaults", path.display())),
            ),
        }
    }

    /// The configured default engine, falling back to the Rust engine when the
    /// name is unknown or unavailable (see [`Settings::load_with_warning`]).
    pub fn parse_engine(&self) -> EngineKind {
        EngineKind::from_name(&self.default_engine).unwrap_or(EngineKind::RustRegex)
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("rgx").join("config.toml"))
}
