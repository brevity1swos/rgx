use crate::engine::{self, CompiledRegex, EngineFlags, EngineKind, RegexEngine};
use crate::explain::{self, ExplainNode};
use crate::input::editor::Editor;

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

pub struct App {
    pub regex_editor: Editor,
    pub test_editor: Editor,
    pub replace_editor: Editor,
    pub focused_panel: u8, // 0=regex, 1=test, 2=replace, 3=matches, 4=explanation
    pub engine_kind: EngineKind,
    pub flags: EngineFlags,
    pub matches: Vec<engine::Match>,
    pub replace_result: Option<engine::ReplaceResult>,
    pub explanation: Vec<ExplainNode>,
    pub error: Option<String>,
    pub show_help: bool,
    pub help_page: usize,
    pub should_quit: bool,
    pub match_scroll: u16,
    pub replace_scroll: u16,
    pub explain_scroll: u16,
    // Pattern history
    pub pattern_history: Vec<String>,
    pub history_index: Option<usize>,
    history_temp: Option<String>,
    // Match selection + clipboard
    pub selected_match: usize,
    pub selected_capture: Option<usize>,
    pub clipboard_status: Option<String>,
    engine: Box<dyn RegexEngine>,
    compiled: Option<Box<dyn CompiledRegex>>,
}

impl App {
    pub fn new(engine_kind: EngineKind, flags: EngineFlags) -> Self {
        let engine = engine::create_engine(engine_kind);
        Self {
            regex_editor: Editor::new(),
            test_editor: Editor::new(),
            replace_editor: Editor::new(),
            focused_panel: 0,
            engine_kind,
            flags,
            matches: Vec::new(),
            replace_result: None,
            explanation: Vec::new(),
            error: None,
            show_help: false,
            help_page: 0,
            should_quit: false,
            match_scroll: 0,
            replace_scroll: 0,
            explain_scroll: 0,
            pattern_history: Vec::new(),
            history_index: None,
            history_temp: None,
            selected_match: 0,
            selected_capture: None,
            clipboard_status: None,
            engine,
            compiled: None,
        }
    }

    pub fn set_replacement(&mut self, text: &str) {
        self.replace_editor = Editor::with_content(text.to_string());
        self.rereplace();
    }

    pub fn scroll_replace_up(&mut self) {
        self.replace_scroll = self.replace_scroll.saturating_sub(1);
    }

    pub fn scroll_replace_down(&mut self) {
        self.replace_scroll = self.replace_scroll.saturating_add(1);
    }

    pub fn rereplace(&mut self) {
        let template = self.replace_editor.content().to_string();
        if template.is_empty() || self.matches.is_empty() {
            self.replace_result = None;
            return;
        }
        let text = self.test_editor.content().to_string();
        self.replace_result = Some(engine::replace_all(&text, &self.matches, &template));
    }

    pub fn set_pattern(&mut self, pattern: &str) {
        self.regex_editor = Editor::with_content(pattern.to_string());
        self.recompute();
    }

    pub fn set_test_string(&mut self, text: &str) {
        self.test_editor = Editor::with_content(text.to_string());
        self.rematch();
    }

    pub fn switch_engine(&mut self) {
        self.engine_kind = self.engine_kind.next();
        self.engine = engine::create_engine(self.engine_kind);
        self.recompute();
    }

    pub fn scroll_match_up(&mut self) {
        self.match_scroll = self.match_scroll.saturating_sub(1);
    }

    pub fn scroll_match_down(&mut self) {
        self.match_scroll = self.match_scroll.saturating_add(1);
    }

    pub fn scroll_explain_up(&mut self) {
        self.explain_scroll = self.explain_scroll.saturating_sub(1);
    }

    pub fn scroll_explain_down(&mut self) {
        self.explain_scroll = self.explain_scroll.saturating_add(1);
    }

    pub fn recompute(&mut self) {
        let pattern = self.regex_editor.content().to_string();
        self.match_scroll = 0;
        self.explain_scroll = 0;

        if pattern.is_empty() {
            self.compiled = None;
            self.matches.clear();
            self.explanation.clear();
            self.error = None;
            return;
        }

        // Compile
        match self.engine.compile(&pattern, &self.flags) {
            Ok(compiled) => {
                self.compiled = Some(compiled);
                self.error = None;
            }
            Err(e) => {
                self.compiled = None;
                self.matches.clear();
                self.error = Some(e.to_string());
            }
        }

        // Explain (uses regex-syntax, independent of engine)
        match explain::explain(&pattern) {
            Ok(nodes) => self.explanation = nodes,
            Err(e) => {
                self.explanation.clear();
                if self.error.is_none() {
                    self.error = Some(e);
                }
            }
        }

        // Match
        self.rematch();
    }

    pub fn rematch(&mut self) {
        self.match_scroll = 0;
        self.selected_match = 0;
        self.selected_capture = None;
        if let Some(compiled) = &self.compiled {
            let text = self.test_editor.content().to_string();
            if text.is_empty() {
                self.matches.clear();
                self.replace_result = None;
                return;
            }
            match compiled.find_matches(&text) {
                Ok(m) => self.matches = m,
                Err(e) => {
                    self.matches.clear();
                    self.error = Some(e.to_string());
                }
            }
        } else {
            self.matches.clear();
        }
        self.rereplace();
    }

    // --- Pattern history ---

    pub fn commit_pattern_to_history(&mut self) {
        let pattern = self.regex_editor.content().to_string();
        if pattern.is_empty() {
            return;
        }
        if self.pattern_history.last().map(|s| s.as_str()) == Some(&pattern) {
            return;
        }
        self.pattern_history.push(pattern);
        if self.pattern_history.len() > 100 {
            self.pattern_history.remove(0);
        }
        self.history_index = None;
        self.history_temp = None;
    }

    pub fn history_prev(&mut self) {
        if self.pattern_history.is_empty() {
            return;
        }
        let new_index = match self.history_index {
            Some(0) => return,
            Some(idx) => idx - 1,
            None => {
                self.history_temp = Some(self.regex_editor.content().to_string());
                self.pattern_history.len() - 1
            }
        };
        self.history_index = Some(new_index);
        let pattern = self.pattern_history[new_index].clone();
        self.regex_editor = Editor::with_content(pattern);
        self.recompute();
    }

    pub fn history_next(&mut self) {
        let idx = match self.history_index {
            Some(idx) => idx,
            None => return,
        };
        if idx + 1 < self.pattern_history.len() {
            let new_index = idx + 1;
            self.history_index = Some(new_index);
            let pattern = self.pattern_history[new_index].clone();
            self.regex_editor = Editor::with_content(pattern);
            self.recompute();
        } else {
            // Past end — restore temp
            self.history_index = None;
            let content = self.history_temp.take().unwrap_or_default();
            self.regex_editor = Editor::with_content(content);
            self.recompute();
        }
    }

    // --- Match selection + clipboard ---

    pub fn select_match_next(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        match self.selected_capture {
            None => {
                let m = &self.matches[self.selected_match];
                if !m.captures.is_empty() {
                    self.selected_capture = Some(0);
                } else if self.selected_match + 1 < self.matches.len() {
                    self.selected_match += 1;
                }
            }
            Some(ci) => {
                let m = &self.matches[self.selected_match];
                if ci + 1 < m.captures.len() {
                    self.selected_capture = Some(ci + 1);
                } else if self.selected_match + 1 < self.matches.len() {
                    self.selected_match += 1;
                    self.selected_capture = None;
                }
            }
        }
        self.scroll_to_selected();
    }

    pub fn select_match_prev(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        match self.selected_capture {
            Some(0) => {
                self.selected_capture = None;
            }
            Some(ci) => {
                self.selected_capture = Some(ci - 1);
            }
            None => {
                if self.selected_match > 0 {
                    self.selected_match -= 1;
                    let m = &self.matches[self.selected_match];
                    if !m.captures.is_empty() {
                        self.selected_capture = Some(m.captures.len() - 1);
                    }
                }
            }
        }
        self.scroll_to_selected();
    }

    fn scroll_to_selected(&mut self) {
        // Calculate the line index of the selected item
        let mut line = 0usize;
        for i in 0..self.selected_match {
            line += 1 + self.matches[i].captures.len();
        }
        if let Some(ci) = self.selected_capture {
            line += 1 + ci;
        }
        self.match_scroll = line as u16;
    }

    pub fn copy_selected_match(&mut self) {
        let text = self.selected_text();
        let Some(text) = text else { return };
        match arboard::Clipboard::new() {
            Ok(mut cb) => match cb.set_text(&text) {
                Ok(()) => {
                    self.clipboard_status = Some(format!("Copied: \"{}\"", truncate(&text, 40)));
                }
                Err(e) => {
                    self.clipboard_status = Some(format!("Clipboard error: {e}"));
                }
            },
            Err(e) => {
                self.clipboard_status = Some(format!("Clipboard error: {e}"));
            }
        }
    }

    fn selected_text(&self) -> Option<String> {
        let m = self.matches.get(self.selected_match)?;
        match self.selected_capture {
            None => Some(m.text.clone()),
            Some(ci) => m.captures.get(ci).map(|c| c.text.clone()),
        }
    }
}
