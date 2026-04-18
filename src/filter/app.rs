//! TUI-mode state for `rgx filter`.

use crate::engine::{self, CompiledRegex, EngineFlags, EngineKind, RegexEngine};
use crate::filter::FilterOptions;
use crate::input::editor::Editor;

pub struct FilterApp {
    pub pattern_editor: Editor,
    pub options: FilterOptions,
    pub lines: Vec<String>,
    /// Optional per-line extracted strings when the user passed `--json`.
    /// Same length as `lines`. `None` at index `i` means line `i` should be
    /// skipped (JSON parse failure, missing path, or non-string value). When
    /// `json_extracted` is `None`, matching runs against the raw lines.
    pub json_extracted: Option<Vec<Option<String>>>,
    /// Indices of `lines` that currently match the pattern.
    pub matched: Vec<usize>,
    /// Byte ranges within each matched *input* that the pattern matched.
    /// In `--json` mode these are spans within the extracted string, not the
    /// raw line. Length equals `matched.len()`; empty per-line in invert mode.
    pub match_spans: Vec<Vec<std::ops::Range<usize>>>,
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
        Self::build(lines, None, initial_pattern, options)
    }

    /// Construct a filter app whose matching runs against pre-extracted JSON
    /// field values (from the `--json` flag). `extracted[i]` is `Some(s)` when
    /// line `i` parsed and yielded a string value; `None` otherwise.
    pub fn with_json_extracted(
        lines: Vec<String>,
        extracted: Vec<Option<String>>,
        initial_pattern: &str,
        options: FilterOptions,
    ) -> Self {
        assert_eq!(
            lines.len(),
            extracted.len(),
            "extracted length must match lines length"
        );
        Self::build(lines, Some(extracted), initial_pattern, options)
    }

    fn build(
        lines: Vec<String>,
        json_extracted: Option<Vec<Option<String>>>,
        initial_pattern: &str,
        options: FilterOptions,
    ) -> Self {
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
            json_extracted,
            matched: Vec::new(),
            match_spans: Vec::new(),
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
            // Empty pattern: every line with a "present" input passes (iff not
            // inverted). In --json mode, lines whose extracted value is None
            // are excluded regardless of invert.
            self.matched = if let Some(extracted) = self.json_extracted.as_ref() {
                extracted
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, v)| {
                        if v.is_some() && !self.options.invert {
                            Some(idx)
                        } else {
                            None
                        }
                    })
                    .collect()
            } else if self.options.invert {
                Vec::new()
            } else {
                (0..self.lines.len()).collect()
            };
            // Nothing to highlight with an empty pattern.
            self.match_spans = vec![Vec::new(); self.matched.len()];
            self.clamp_selection();
            return;
        }
        match self.engine.compile(&pattern, &self.engine_flags) {
            Ok(compiled) => {
                let (indices, spans) = self.collect_matches(&*compiled);
                self.matched = indices;
                self.match_spans = spans;
                self.clamp_selection();
            }
            Err(err) => {
                self.error = Some(err.to_string());
                self.matched.clear();
                self.match_spans.clear();
                self.selected = 0;
                self.scroll = 0;
            }
        }
    }

    fn collect_matches(
        &self,
        compiled: &dyn CompiledRegex,
    ) -> (Vec<usize>, Vec<Vec<std::ops::Range<usize>>>) {
        let mut indices = Vec::with_capacity(self.lines.len());
        let mut all_spans = Vec::with_capacity(self.lines.len());
        for idx in 0..self.lines.len() {
            // In --json mode we match against the extracted field, not the
            // raw line. None extracted values never match (and are excluded
            // from invert-mode output too).
            let haystack: &str = if let Some(extracted) = self.json_extracted.as_ref() {
                match &extracted[idx] {
                    Some(s) => s.as_str(),
                    None => continue,
                }
            } else {
                &self.lines[idx]
            };
            let line_matches = compiled.find_matches(haystack).unwrap_or_default();
            let hit = !line_matches.is_empty();
            if hit != self.options.invert {
                indices.push(idx);
                // In invert mode we emit lines that did NOT match — no spans
                // to highlight per the task spec.
                if self.options.invert {
                    all_spans.push(Vec::new());
                } else {
                    all_spans.push(line_matches.into_iter().map(|m| m.start..m.end).collect());
                }
            }
        }
        (indices, all_spans)
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
