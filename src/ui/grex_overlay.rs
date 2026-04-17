//! grex overlay widget — lets users enter example strings and load a generated regex into the main editor.

use std::time::Instant;

use crate::grex_integration::GrexOptions;
use crate::input::editor::Editor;

pub struct GrexOverlayState {
    pub editor: Editor,
    pub options: GrexOptions,
    pub generated_pattern: Option<String>,
    pub generation_counter: u64,
    pub debounce_deadline: Option<Instant>,
}

impl Default for GrexOverlayState {
    fn default() -> Self {
        Self {
            editor: Editor::new(),
            options: GrexOptions::default(),
            generated_pattern: None,
            generation_counter: 0,
            debounce_deadline: None,
        }
    }
}
