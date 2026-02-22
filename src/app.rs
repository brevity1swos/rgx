use crate::engine::{self, CompiledRegex, EngineFlags, EngineKind, RegexEngine};
use crate::explain::{self, ExplainNode};
use crate::input::editor::Editor;

pub struct App {
    pub regex_editor: Editor,
    pub test_editor: Editor,
    pub focused_panel: u8, // 0 = regex, 1 = test, 2 = matches, 3 = explanation
    pub engine_kind: EngineKind,
    pub flags: EngineFlags,
    pub matches: Vec<engine::Match>,
    pub explanation: Vec<ExplainNode>,
    pub error: Option<String>,
    pub show_help: bool,
    pub should_quit: bool,
    pub match_scroll: u16,
    pub explain_scroll: u16,
    engine: Box<dyn RegexEngine>,
    compiled: Option<Box<dyn CompiledRegex>>,
}

impl App {
    pub fn new(engine_kind: EngineKind, flags: EngineFlags) -> Self {
        let engine = engine::create_engine(engine_kind);
        Self {
            regex_editor: Editor::new(),
            test_editor: Editor::new(),
            focused_panel: 0,
            engine_kind,
            flags,
            matches: Vec::new(),
            explanation: Vec::new(),
            error: None,
            show_help: false,
            should_quit: false,
            match_scroll: 0,
            explain_scroll: 0,
            engine,
            compiled: None,
        }
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
        if let Some(compiled) = &self.compiled {
            let text = self.test_editor.content().to_string();
            if text.is_empty() {
                self.matches.clear();
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
    }
}
