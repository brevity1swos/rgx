//! TUI-mode state for `rgx filter`.

use crate::engine::{self, CompiledRegex, EngineFlags, EngineKind, RegexEngine};
use crate::filter::FilterOptions;
use crate::input::editor::Editor;

pub struct FilterApp {
    pub pattern_editor: Editor,
    pub options: FilterOptions,
    pub lines: Vec<String>,
    /// Indices of `lines` that currently match the pattern.
    pub matched: Vec<usize>,
    /// Selected index into `matched` for the cursor in the match list.
    pub selected: usize,
    /// Scroll offset (first visible index into `matched`).
    pub scroll: usize,
    /// Compilation error from the last `recompute`, if any.
    pub error: Option<String>,
    /// Whether to quit the event loop on next tick.
    pub should_quit: bool,
    /// Outcome decided by the user: emit the filtered output, or discard.
    pub outcome: Outcome,
    engine: Box<dyn RegexEngine>,
    engine_flags: EngineFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Pending,
    Emit,
    Discard,
}

impl FilterApp {
    pub fn new(lines: Vec<String>, initial_pattern: &str, options: FilterOptions) -> Self {
        let pattern_editor = Editor::with_content(initial_pattern.to_string());
        let engine_flags = EngineFlags {
            case_insensitive: options.case_insensitive,
            ..EngineFlags::default()
        };
        let engine = engine::create_engine(EngineKind::RustRegex);
        let mut app = Self {
            pattern_editor,
            options,
            lines,
            matched: Vec::new(),
            selected: 0,
            scroll: 0,
            error: None,
            should_quit: false,
            outcome: Outcome::Pending,
            engine,
            engine_flags,
        };
        app.recompute();
        app
    }

    pub fn pattern(&self) -> &str {
        self.pattern_editor.content()
    }

    pub fn recompute(&mut self) {
        self.error = None;
        let pattern = self.pattern().to_string();
        if pattern.is_empty() {
            self.matched = if self.options.invert {
                Vec::new()
            } else {
                (0..self.lines.len()).collect()
            };
            self.clamp_selection();
            return;
        }
        match self.engine.compile(&pattern, &self.engine_flags) {
            Ok(compiled) => {
                self.matched = self.collect_matches(&*compiled);
                self.clamp_selection();
            }
            Err(err) => {
                self.error = Some(err.to_string());
                self.matched.clear();
                self.selected = 0;
                self.scroll = 0;
            }
        }
    }

    fn collect_matches(&self, compiled: &dyn CompiledRegex) -> Vec<usize> {
        let mut out = Vec::with_capacity(self.lines.len());
        for (idx, line) in self.lines.iter().enumerate() {
            let hit = compiled
                .find_matches(line)
                .map(|v| !v.is_empty())
                .unwrap_or(false);
            if hit != self.options.invert {
                out.push(idx);
            }
        }
        out
    }

    fn clamp_selection(&mut self) {
        if self.matched.is_empty() {
            self.selected = 0;
            self.scroll = 0;
        } else if self.selected >= self.matched.len() {
            self.selected = self.matched.len() - 1;
        }
    }

    pub fn select_next(&mut self) {
        if self.selected + 1 < self.matched.len() {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn toggle_case_insensitive(&mut self) {
        self.options.case_insensitive = !self.options.case_insensitive;
        self.engine_flags.case_insensitive = self.options.case_insensitive;
        self.recompute();
    }

    pub fn toggle_invert(&mut self) {
        self.options.invert = !self.options.invert;
        self.recompute();
    }
}
