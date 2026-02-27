use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::app::App;
use crate::engine::EngineKind;

#[derive(Serialize, Deserialize)]
pub struct Workspace {
    pub pattern: String,
    pub test_string: String,
    pub replacement: String,
    pub engine: String,
    pub case_insensitive: bool,
    pub multiline: bool,
    pub dotall: bool,
    pub unicode: bool,
    pub extended: bool,
    pub show_whitespace: bool,
}

impl Workspace {
    pub fn from_app(app: &App) -> Self {
        let engine = match app.engine_kind {
            EngineKind::RustRegex => "rust",
            EngineKind::FancyRegex => "fancy",
            #[cfg(feature = "pcre2-engine")]
            EngineKind::Pcre2 => "pcre2",
        };
        Self {
            pattern: app.regex_editor.content().to_string(),
            test_string: app.test_editor.content().to_string(),
            replacement: app.replace_editor.content().to_string(),
            engine: engine.to_string(),
            case_insensitive: app.flags.case_insensitive,
            multiline: app.flags.multi_line,
            dotall: app.flags.dot_matches_newline,
            unicode: app.flags.unicode,
            extended: app.flags.extended,
            show_whitespace: app.show_whitespace,
        }
    }

    pub fn apply(&self, app: &mut App) {
        let engine_kind = match self.engine.as_str() {
            "fancy" => EngineKind::FancyRegex,
            #[cfg(feature = "pcre2-engine")]
            "pcre2" => EngineKind::Pcre2,
            _ => EngineKind::RustRegex,
        };
        if app.engine_kind != engine_kind {
            app.engine_kind = engine_kind;
            app.switch_engine_to(engine_kind);
        }
        app.flags.case_insensitive = self.case_insensitive;
        app.flags.multi_line = self.multiline;
        app.flags.dot_matches_newline = self.dotall;
        app.flags.unicode = self.unicode;
        app.flags.extended = self.extended;
        app.show_whitespace = self.show_whitespace;
        app.set_test_string(&self.test_string);
        if !self.replacement.is_empty() {
            app.set_replacement(&self.replacement);
        }
        app.set_pattern(&self.pattern);
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let ws: Self = toml::from_str(&content)?;
        Ok(ws)
    }
}
