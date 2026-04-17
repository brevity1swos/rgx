use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::ansi::{GREEN_BOLD, RED_BOLD, RESET};
use crate::engine::{self, CompiledRegex, EngineFlags, EngineKind, RegexEngine};
use crate::explain::{self, ExplainNode};
use crate::input::editor::Editor;
use crate::input::Action;

const MAX_PATTERN_HISTORY: usize = 100;
const STATUS_DISPLAY_TICKS: u32 = 40; // ~2 seconds at 50ms tick rate

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub engine: EngineKind,
    pub compile_time: Duration,
    pub match_time: Duration,
    pub match_count: usize,
    pub error: Option<String>,
}

fn truncate(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        s.to_string()
    } else {
        let end = s
            .char_indices()
            .nth(max_chars)
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        format!("{}...", &s[..end])
    }
}

#[derive(Default)]
pub struct OverlayState {
    pub help: bool,
    pub help_page: usize,
    pub recipes: bool,
    pub recipe_index: usize,
    pub benchmark: bool,
    pub codegen: bool,
    pub codegen_language_index: usize,
    pub grex: Option<crate::ui::grex_overlay::GrexOverlayState>,
}

#[derive(Default)]
pub struct ScrollState {
    pub match_scroll: u16,
    pub replace_scroll: u16,
    pub explain_scroll: u16,
}

#[derive(Default)]
pub struct PatternHistory {
    pub entries: VecDeque<String>,
    pub index: Option<usize>,
    pub temp: Option<String>,
}

#[derive(Default)]
pub struct MatchSelection {
    pub match_index: usize,
    pub capture_index: Option<usize>,
}

#[derive(Default)]
pub struct StatusMessage {
    pub text: Option<String>,
    ticks: u32,
}

impl StatusMessage {
    pub fn set(&mut self, message: String) {
        self.text = Some(message);
        self.ticks = STATUS_DISPLAY_TICKS;
    }

    pub fn tick(&mut self) -> bool {
        if self.text.is_some() {
            if self.ticks > 0 {
                self.ticks -= 1;
            } else {
                self.text = None;
                return true;
            }
        }
        false
    }
}

pub struct App {
    pub regex_editor: Editor,
    pub test_editor: Editor,
    pub replace_editor: Editor,
    pub focused_panel: u8,
    pub engine_kind: EngineKind,
    pub flags: EngineFlags,
    pub matches: Vec<engine::Match>,
    pub replace_result: Option<engine::ReplaceResult>,
    pub explanation: Vec<ExplainNode>,
    pub error: Option<String>,
    pub overlay: OverlayState,
    pub should_quit: bool,
    pub scroll: ScrollState,
    pub history: PatternHistory,
    pub selection: MatchSelection,
    pub status: StatusMessage,
    pub show_whitespace: bool,
    pub rounded_borders: bool,
    pub vim_mode: bool,
    pub vim_state: crate::input::vim::VimState,
    pub compile_time: Option<Duration>,
    pub match_time: Option<Duration>,
    pub error_offset: Option<usize>,
    pub output_on_quit: bool,
    pub workspace_path: Option<String>,
    pub benchmark_results: Vec<BenchmarkResult>,
    pub syntax_tokens: Vec<crate::ui::syntax_highlight::SyntaxToken>,
    #[cfg(feature = "pcre2-engine")]
    pub debug_session: Option<crate::engine::pcre2_debug::DebugSession>,
    #[cfg(feature = "pcre2-engine")]
    debug_cache: Option<crate::engine::pcre2_debug::DebugSession>,
    engine: Box<dyn RegexEngine>,
    compiled: Option<Box<dyn CompiledRegex>>,
}

impl App {
    pub const PANEL_REGEX: u8 = 0;
    pub const PANEL_TEST: u8 = 1;
    pub const PANEL_REPLACE: u8 = 2;
    pub const PANEL_MATCHES: u8 = 3;
    pub const PANEL_EXPLAIN: u8 = 4;
    pub const PANEL_COUNT: u8 = 5;
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
            overlay: OverlayState::default(),
            should_quit: false,
            scroll: ScrollState::default(),
            history: PatternHistory::default(),
            selection: MatchSelection::default(),
            status: StatusMessage::default(),
            show_whitespace: false,
            rounded_borders: false,
            vim_mode: false,
            vim_state: crate::input::vim::VimState::new(),
            compile_time: None,
            match_time: None,
            error_offset: None,
            output_on_quit: false,
            workspace_path: None,
            benchmark_results: Vec::new(),
            syntax_tokens: Vec::new(),
            #[cfg(feature = "pcre2-engine")]
            debug_session: None,
            #[cfg(feature = "pcre2-engine")]
            debug_cache: None,
            engine,
            compiled: None,
        }
    }

    pub fn set_replacement(&mut self, text: &str) {
        self.replace_editor = Editor::with_content(text.to_string());
        self.rereplace();
    }

    pub fn scroll_replace_up(&mut self) {
        self.scroll.replace_scroll = self.scroll.replace_scroll.saturating_sub(1);
    }

    pub fn scroll_replace_down(&mut self) {
        self.scroll.replace_scroll = self.scroll.replace_scroll.saturating_add(1);
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

    /// Low-level engine setter. Does NOT call `recompute()` — the caller
    /// must trigger recompilation separately if needed.
    pub fn switch_engine_to(&mut self, kind: EngineKind) {
        self.engine_kind = kind;
        self.engine = engine::create_engine(kind);
    }

    pub fn scroll_match_up(&mut self) {
        self.scroll.match_scroll = self.scroll.match_scroll.saturating_sub(1);
    }

    pub fn scroll_match_down(&mut self) {
        self.scroll.match_scroll = self.scroll.match_scroll.saturating_add(1);
    }

    pub fn scroll_explain_up(&mut self) {
        self.scroll.explain_scroll = self.scroll.explain_scroll.saturating_sub(1);
    }

    pub fn scroll_explain_down(&mut self) {
        self.scroll.explain_scroll = self.scroll.explain_scroll.saturating_add(1);
    }

    pub fn recompute(&mut self) {
        let pattern = self.regex_editor.content().to_string();
        self.scroll.match_scroll = 0;
        self.scroll.explain_scroll = 0;
        self.error_offset = None;

        if pattern.is_empty() {
            self.compiled = None;
            self.matches.clear();
            self.explanation.clear();
            self.error = None;
            self.compile_time = None;
            self.match_time = None;
            self.syntax_tokens.clear();
            return;
        }

        // Auto-select engine: upgrade (never downgrade) if the pattern
        // requires a more powerful engine than the currently active one.
        let suggested = engine::detect_minimum_engine(&pattern);
        if engine::is_engine_upgrade(self.engine_kind, suggested) {
            let prev = self.engine_kind;
            self.engine_kind = suggested;
            self.engine = engine::create_engine(suggested);
            self.status.set(format!(
                "Auto-switched {} \u{2192} {} for this pattern",
                prev, suggested,
            ));
        }

        // Compile
        let compile_start = Instant::now();
        match self.engine.compile(&pattern, &self.flags) {
            Ok(compiled) => {
                self.compile_time = Some(compile_start.elapsed());
                self.compiled = Some(compiled);
                self.error = None;
            }
            Err(e) => {
                self.compile_time = Some(compile_start.elapsed());
                self.compiled = None;
                self.matches.clear();
                self.error = Some(e.to_string());
            }
        }

        // Rebuild syntax highlight tokens (pattern changed)
        self.syntax_tokens = crate::ui::syntax_highlight::highlight(&pattern);

        // Explain (uses regex-syntax, independent of engine)
        match explain::explain(&pattern) {
            Ok(nodes) => self.explanation = nodes,
            Err((msg, offset)) => {
                self.explanation.clear();
                if self.error_offset.is_none() {
                    self.error_offset = offset;
                }
                if self.error.is_none() {
                    self.error = Some(msg);
                }
            }
        }

        // Match
        self.rematch();
    }

    pub fn rematch(&mut self) {
        self.scroll.match_scroll = 0;
        self.selection.match_index = 0;
        self.selection.capture_index = None;
        if let Some(compiled) = &self.compiled {
            let text = self.test_editor.content().to_string();
            if text.is_empty() {
                self.matches.clear();
                self.replace_result = None;
                self.match_time = None;
                return;
            }
            let match_start = Instant::now();
            match compiled.find_matches(&text) {
                Ok(m) => {
                    self.match_time = Some(match_start.elapsed());
                    self.matches = m;
                }
                Err(e) => {
                    self.match_time = Some(match_start.elapsed());
                    self.matches.clear();
                    self.error = Some(e.to_string());
                }
            }
        } else {
            self.matches.clear();
            self.match_time = None;
        }
        self.rereplace();
    }

    // --- Pattern history ---

    pub fn commit_pattern_to_history(&mut self) {
        let pattern = self.regex_editor.content().to_string();
        if pattern.is_empty() {
            return;
        }
        if self.history.entries.back().map(|s| s.as_str()) == Some(&pattern) {
            return;
        }
        self.history.entries.push_back(pattern);
        if self.history.entries.len() > MAX_PATTERN_HISTORY {
            self.history.entries.pop_front();
        }
        self.history.index = None;
        self.history.temp = None;
    }

    pub fn history_prev(&mut self) {
        if self.history.entries.is_empty() {
            return;
        }
        let new_index = match self.history.index {
            Some(0) => return,
            Some(idx) => idx - 1,
            None => {
                self.history.temp = Some(self.regex_editor.content().to_string());
                self.history.entries.len() - 1
            }
        };
        self.history.index = Some(new_index);
        let pattern = self.history.entries[new_index].clone();
        self.regex_editor = Editor::with_content(pattern);
        self.recompute();
    }

    pub fn history_next(&mut self) {
        let idx = match self.history.index {
            Some(idx) => idx,
            None => return,
        };
        if idx + 1 < self.history.entries.len() {
            let new_index = idx + 1;
            self.history.index = Some(new_index);
            let pattern = self.history.entries[new_index].clone();
            self.regex_editor = Editor::with_content(pattern);
            self.recompute();
        } else {
            // Past end — restore temp
            self.history.index = None;
            let content = self.history.temp.take().unwrap_or_default();
            self.regex_editor = Editor::with_content(content);
            self.recompute();
        }
    }

    // --- Match selection + clipboard ---

    pub fn select_match_next(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        match self.selection.capture_index {
            None => {
                let m = &self.matches[self.selection.match_index];
                if !m.captures.is_empty() {
                    self.selection.capture_index = Some(0);
                } else if self.selection.match_index + 1 < self.matches.len() {
                    self.selection.match_index += 1;
                }
            }
            Some(ci) => {
                let m = &self.matches[self.selection.match_index];
                if ci + 1 < m.captures.len() {
                    self.selection.capture_index = Some(ci + 1);
                } else if self.selection.match_index + 1 < self.matches.len() {
                    self.selection.match_index += 1;
                    self.selection.capture_index = None;
                }
            }
        }
        self.scroll_to_selected();
    }

    pub fn select_match_prev(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        match self.selection.capture_index {
            Some(0) => {
                self.selection.capture_index = None;
            }
            Some(ci) => {
                self.selection.capture_index = Some(ci - 1);
            }
            None => {
                if self.selection.match_index > 0 {
                    self.selection.match_index -= 1;
                    let m = &self.matches[self.selection.match_index];
                    if !m.captures.is_empty() {
                        self.selection.capture_index = Some(m.captures.len() - 1);
                    }
                }
            }
        }
        self.scroll_to_selected();
    }

    fn scroll_to_selected(&mut self) {
        if self.matches.is_empty() || self.selection.match_index >= self.matches.len() {
            return;
        }
        let mut line = 0usize;
        for i in 0..self.selection.match_index {
            line += 1 + self.matches[i].captures.len();
        }
        if let Some(ci) = self.selection.capture_index {
            line += 1 + ci;
        }
        self.scroll.match_scroll = u16::try_from(line).unwrap_or(u16::MAX);
    }

    pub fn copy_selected_match(&mut self) {
        let text = self.selected_text();
        let Some(text) = text else { return };
        let msg = format!("Copied: \"{}\"", truncate(&text, 40));
        self.copy_to_clipboard(&text, &msg);
    }

    fn copy_to_clipboard(&mut self, text: &str, success_msg: &str) {
        match arboard::Clipboard::new() {
            Ok(mut cb) => match cb.set_text(text) {
                Ok(()) => self.status.set(success_msg.to_string()),
                Err(e) => self.status.set(format!("Clipboard error: {e}")),
            },
            Err(e) => self.status.set(format!("Clipboard error: {e}")),
        }
    }

    /// Print match results or replacement output to stdout.
    pub fn print_output(&self, group: Option<&str>, count: bool, color: bool) {
        if count {
            println!("{}", self.matches.len());
            return;
        }
        if let Some(ref result) = self.replace_result {
            if color {
                print_colored_replace(&result.output, &result.segments);
            } else {
                print!("{}", result.output);
            }
        } else if let Some(group_spec) = group {
            for m in &self.matches {
                if let Some(text) = engine::lookup_capture(m, group_spec) {
                    if color {
                        println!("{RED_BOLD}{text}{RESET}");
                    } else {
                        println!("{text}");
                    }
                } else {
                    eprintln!("rgx: group '{group_spec}' not found in match");
                }
            }
        } else if color {
            let text = self.test_editor.content();
            print_colored_matches(text, &self.matches);
        } else {
            for m in &self.matches {
                println!("{}", m.text);
            }
        }
    }

    /// Print matches as structured JSON.
    pub fn print_json_output(&self) {
        println!(
            "{}",
            serde_json::to_string_pretty(&self.matches).unwrap_or_else(|_| "[]".to_string())
        );
    }

    fn selected_text(&self) -> Option<String> {
        let m = self.matches.get(self.selection.match_index)?;
        match self.selection.capture_index {
            None => Some(m.text.clone()),
            Some(ci) => m.captures.get(ci).map(|c| c.text.clone()),
        }
    }

    /// Apply a mutating editor operation to the currently focused editor panel,
    /// then trigger the appropriate recompute/rematch/rereplace.
    pub fn edit_focused(&mut self, f: impl FnOnce(&mut Editor)) {
        match self.focused_panel {
            Self::PANEL_REGEX => {
                f(&mut self.regex_editor);
                self.recompute();
            }
            Self::PANEL_TEST => {
                f(&mut self.test_editor);
                self.rematch();
            }
            Self::PANEL_REPLACE => {
                f(&mut self.replace_editor);
                self.rereplace();
            }
            _ => {}
        }
    }

    /// Apply a non-mutating cursor movement to the currently focused editor panel.
    pub fn move_focused(&mut self, f: impl FnOnce(&mut Editor)) {
        match self.focused_panel {
            Self::PANEL_REGEX => f(&mut self.regex_editor),
            Self::PANEL_TEST => f(&mut self.test_editor),
            Self::PANEL_REPLACE => f(&mut self.replace_editor),
            _ => {}
        }
    }

    pub fn run_benchmark(&mut self) {
        let pattern = self.regex_editor.content().to_string();
        let text = self.test_editor.content().to_string();
        if pattern.is_empty() || text.is_empty() {
            return;
        }

        let mut results = Vec::new();
        for kind in EngineKind::all() {
            let eng = engine::create_engine(kind);
            let compile_start = Instant::now();
            let compiled = match eng.compile(&pattern, &self.flags) {
                Ok(c) => c,
                Err(e) => {
                    results.push(BenchmarkResult {
                        engine: kind,
                        compile_time: compile_start.elapsed(),
                        match_time: Duration::ZERO,
                        match_count: 0,
                        error: Some(e.to_string()),
                    });
                    continue;
                }
            };
            let compile_time = compile_start.elapsed();
            let match_start = Instant::now();
            let (match_count, error) = match compiled.find_matches(&text) {
                Ok(matches) => (matches.len(), None),
                Err(e) => (0, Some(e.to_string())),
            };
            results.push(BenchmarkResult {
                engine: kind,
                compile_time,
                match_time: match_start.elapsed(),
                match_count,
                error,
            });
        }
        self.benchmark_results = results;
        self.overlay.benchmark = true;
    }

    /// Generate a regex101.com URL from the current state.
    pub fn regex101_url(&self) -> String {
        let pattern = self.regex_editor.content();
        let test_string = self.test_editor.content();

        let flavor = match self.engine_kind {
            #[cfg(feature = "pcre2-engine")]
            EngineKind::Pcre2 => "pcre2",
            _ => "ecmascript",
        };

        let mut flags = String::from("g");
        if self.flags.case_insensitive {
            flags.push('i');
        }
        if self.flags.multi_line {
            flags.push('m');
        }
        if self.flags.dot_matches_newline {
            flags.push('s');
        }
        if self.flags.unicode {
            flags.push('u');
        }
        if self.flags.extended {
            flags.push('x');
        }

        format!(
            "https://regex101.com/?regex={}&testString={}&flags={}&flavor={}",
            url_encode(pattern),
            url_encode(test_string),
            url_encode(&flags),
            flavor,
        )
    }

    /// Copy regex101 URL to clipboard.
    pub fn copy_regex101_url(&mut self) {
        let url = self.regex101_url();
        self.copy_to_clipboard(&url, "regex101 URL copied to clipboard");
    }

    /// Generate code for the current pattern in the given language and copy to clipboard.
    pub fn generate_code(&mut self, lang: &crate::codegen::Language) {
        let pattern = self.regex_editor.content().to_string();
        if pattern.is_empty() {
            self.status
                .set("No pattern to generate code for".to_string());
            return;
        }
        let code = crate::codegen::generate_code(lang, &pattern, &self.flags);
        self.copy_to_clipboard(&code, &format!("{} code copied to clipboard", lang));
        self.overlay.codegen = false;
    }

    #[cfg(feature = "pcre2-engine")]
    pub fn start_debug(&mut self, max_steps: usize) {
        use crate::engine::pcre2_debug::{self, DebugSession};

        let pattern = self.regex_editor.content().to_string();
        let subject = self.test_editor.content().to_string();
        if pattern.is_empty() || subject.is_empty() {
            self.status
                .set("Debugger needs both a pattern and test string".to_string());
            return;
        }

        if self.engine_kind != EngineKind::Pcre2 {
            self.switch_engine_to(EngineKind::Pcre2);
            self.recompute();
        }

        // Restore cached session if pattern and subject haven't changed,
        // preserving the user's step position and heatmap toggle.
        if let Some(ref cached) = self.debug_cache {
            if cached.pattern == pattern && cached.subject == subject {
                self.debug_session = self.debug_cache.take();
                return;
            }
        }

        let start_offset = self.selected_match_start();

        match pcre2_debug::debug_match(&pattern, &subject, &self.flags, max_steps, start_offset) {
            Ok(trace) => {
                self.debug_session = Some(DebugSession {
                    trace,
                    step: 0,
                    show_heatmap: false,
                    pattern,
                    subject,
                });
            }
            Err(e) => {
                self.status.set(format!("Debugger error: {e}"));
            }
        }
    }

    #[cfg(not(feature = "pcre2-engine"))]
    pub fn start_debug(&mut self, _max_steps: usize) {
        self.status
            .set("Debugger requires PCRE2 (build with --features pcre2-engine)".to_string());
    }

    #[cfg(feature = "pcre2-engine")]
    fn selected_match_start(&self) -> usize {
        if !self.matches.is_empty() && self.selection.match_index < self.matches.len() {
            self.matches[self.selection.match_index].start
        } else {
            0
        }
    }

    #[cfg(feature = "pcre2-engine")]
    pub fn close_debug(&mut self) {
        self.debug_cache = self.debug_session.take();
    }

    pub fn debug_step_forward(&mut self) {
        #[cfg(feature = "pcre2-engine")]
        if let Some(ref mut s) = self.debug_session {
            if s.step + 1 < s.trace.steps.len() {
                s.step += 1;
            }
        }
    }

    pub fn debug_step_back(&mut self) {
        #[cfg(feature = "pcre2-engine")]
        if let Some(ref mut s) = self.debug_session {
            s.step = s.step.saturating_sub(1);
        }
    }

    pub fn debug_jump_start(&mut self) {
        #[cfg(feature = "pcre2-engine")]
        if let Some(ref mut s) = self.debug_session {
            s.step = 0;
        }
    }

    pub fn debug_jump_end(&mut self) {
        #[cfg(feature = "pcre2-engine")]
        if let Some(ref mut s) = self.debug_session {
            if !s.trace.steps.is_empty() {
                s.step = s.trace.steps.len() - 1;
            }
        }
    }

    pub fn debug_next_match(&mut self) {
        #[cfg(feature = "pcre2-engine")]
        if let Some(ref mut s) = self.debug_session {
            let current_attempt = s
                .trace
                .steps
                .get(s.step)
                .map(|st| st.match_attempt)
                .unwrap_or(0);
            for (i, step) in s.trace.steps.iter().enumerate().skip(s.step + 1) {
                if step.match_attempt > current_attempt {
                    s.step = i;
                    return;
                }
            }
        }
    }

    pub fn debug_next_backtrack(&mut self) {
        #[cfg(feature = "pcre2-engine")]
        if let Some(ref mut s) = self.debug_session {
            for (i, step) in s.trace.steps.iter().enumerate().skip(s.step + 1) {
                if step.is_backtrack {
                    s.step = i;
                    return;
                }
            }
        }
    }

    pub fn debug_toggle_heatmap(&mut self) {
        #[cfg(feature = "pcre2-engine")]
        if let Some(ref mut s) = self.debug_session {
            s.show_heatmap = !s.show_heatmap;
        }
    }

    pub fn handle_action(&mut self, action: Action, debug_max_steps: usize) {
        match action {
            Action::Quit => {
                self.should_quit = true;
            }
            Action::OutputAndQuit => {
                self.output_on_quit = true;
                self.should_quit = true;
            }
            Action::SwitchPanel => {
                if self.focused_panel == Self::PANEL_REGEX {
                    self.commit_pattern_to_history();
                }
                self.focused_panel = (self.focused_panel + 1) % Self::PANEL_COUNT;
            }
            Action::SwitchPanelBack => {
                if self.focused_panel == Self::PANEL_REGEX {
                    self.commit_pattern_to_history();
                }
                self.focused_panel =
                    (self.focused_panel + Self::PANEL_COUNT - 1) % Self::PANEL_COUNT;
            }
            Action::SwitchEngine => {
                self.switch_engine();
            }
            Action::Undo => {
                if self.focused_panel == Self::PANEL_REGEX && self.regex_editor.undo() {
                    self.recompute();
                } else if self.focused_panel == Self::PANEL_TEST && self.test_editor.undo() {
                    self.rematch();
                } else if self.focused_panel == Self::PANEL_REPLACE && self.replace_editor.undo() {
                    self.rereplace();
                }
            }
            Action::Redo => {
                if self.focused_panel == Self::PANEL_REGEX && self.regex_editor.redo() {
                    self.recompute();
                } else if self.focused_panel == Self::PANEL_TEST && self.test_editor.redo() {
                    self.rematch();
                } else if self.focused_panel == Self::PANEL_REPLACE && self.replace_editor.redo() {
                    self.rereplace();
                }
            }
            Action::HistoryPrev => {
                if self.focused_panel == Self::PANEL_REGEX {
                    self.history_prev();
                }
            }
            Action::HistoryNext => {
                if self.focused_panel == Self::PANEL_REGEX {
                    self.history_next();
                }
            }
            Action::CopyMatch => {
                if self.focused_panel == Self::PANEL_MATCHES {
                    self.copy_selected_match();
                }
            }
            Action::ToggleWhitespace => {
                self.show_whitespace = !self.show_whitespace;
            }
            Action::ToggleCaseInsensitive => {
                self.flags.toggle_case_insensitive();
                self.recompute();
            }
            Action::ToggleMultiLine => {
                self.flags.toggle_multi_line();
                self.recompute();
            }
            Action::ToggleDotAll => {
                self.flags.toggle_dot_matches_newline();
                self.recompute();
            }
            Action::ToggleUnicode => {
                self.flags.toggle_unicode();
                self.recompute();
            }
            Action::ToggleExtended => {
                self.flags.toggle_extended();
                self.recompute();
            }
            Action::ShowHelp => {
                self.overlay.help = true;
            }
            Action::OpenRecipes => {
                self.overlay.recipes = true;
                self.overlay.recipe_index = 0;
            }
            Action::OpenGrex => {
                self.overlay.grex = Some(crate::ui::grex_overlay::GrexOverlayState::default());
            }
            Action::Benchmark => {
                self.run_benchmark();
            }
            Action::ExportRegex101 => {
                self.copy_regex101_url();
            }
            Action::GenerateCode => {
                self.overlay.codegen = true;
                self.overlay.codegen_language_index = 0;
            }
            Action::InsertChar(c) => self.edit_focused(|ed| ed.insert_char(c)),
            Action::InsertNewline => {
                if self.focused_panel == Self::PANEL_TEST {
                    self.test_editor.insert_newline();
                    self.rematch();
                }
            }
            Action::DeleteBack => self.edit_focused(Editor::delete_back),
            Action::DeleteForward => self.edit_focused(Editor::delete_forward),
            Action::MoveCursorLeft => self.move_focused(Editor::move_left),
            Action::MoveCursorRight => self.move_focused(Editor::move_right),
            Action::MoveCursorWordLeft => self.move_focused(Editor::move_word_left),
            Action::MoveCursorWordRight => self.move_focused(Editor::move_word_right),
            Action::ScrollUp => match self.focused_panel {
                Self::PANEL_TEST => self.test_editor.move_up(),
                Self::PANEL_MATCHES => self.select_match_prev(),
                Self::PANEL_EXPLAIN => self.scroll_explain_up(),
                _ => {}
            },
            Action::ScrollDown => match self.focused_panel {
                Self::PANEL_TEST => self.test_editor.move_down(),
                Self::PANEL_MATCHES => self.select_match_next(),
                Self::PANEL_EXPLAIN => self.scroll_explain_down(),
                _ => {}
            },
            Action::MoveCursorHome => self.move_focused(Editor::move_home),
            Action::MoveCursorEnd => self.move_focused(Editor::move_end),
            Action::DeleteCharAtCursor => self.edit_focused(Editor::delete_char_at_cursor),
            Action::DeleteLine => self.edit_focused(Editor::delete_line),
            Action::ChangeLine => self.edit_focused(Editor::clear_line),
            Action::OpenLineBelow => {
                if self.focused_panel == Self::PANEL_TEST {
                    self.test_editor.open_line_below();
                    self.rematch();
                } else {
                    self.vim_state.cancel_insert();
                }
            }
            Action::OpenLineAbove => {
                if self.focused_panel == Self::PANEL_TEST {
                    self.test_editor.open_line_above();
                    self.rematch();
                } else {
                    self.vim_state.cancel_insert();
                }
            }
            Action::MoveToFirstNonBlank => self.move_focused(Editor::move_to_first_non_blank),
            Action::MoveToFirstLine => self.move_focused(Editor::move_to_first_line),
            Action::MoveToLastLine => self.move_focused(Editor::move_to_last_line),
            Action::MoveCursorWordForwardEnd => self.move_focused(Editor::move_word_forward_end),
            Action::EnterInsertMode => {}
            Action::EnterInsertModeAppend => self.move_focused(Editor::move_right),
            Action::EnterInsertModeLineStart => self.move_focused(Editor::move_to_first_non_blank),
            Action::EnterInsertModeLineEnd => self.move_focused(Editor::move_end),
            Action::EnterNormalMode => self.move_focused(Editor::move_left_in_line),
            Action::PasteClipboard => {
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    if let Ok(text) = cb.get_text() {
                        self.edit_focused(|ed| ed.insert_str(&text));
                    }
                }
            }
            Action::ToggleDebugger => {
                #[cfg(feature = "pcre2-engine")]
                if self.debug_session.is_some() {
                    self.close_debug();
                } else {
                    self.start_debug(debug_max_steps);
                }
                #[cfg(not(feature = "pcre2-engine"))]
                self.start_debug(debug_max_steps);
            }
            Action::SaveWorkspace | Action::None => {}
        }
    }

    /// Dispatch a key event to the grex overlay. Returns true if the key was consumed.
    /// Caller should only invoke this when `self.overlay.grex.is_some()`.
    pub fn dispatch_grex_overlay_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::{KeyCode, KeyModifiers};
        const DEBOUNCE_MS: u64 = 150;
        let debounce = std::time::Duration::from_millis(DEBOUNCE_MS);

        let Some(overlay) = self.overlay.grex.as_mut() else {
            return false;
        };

        // Accept / cancel first — these take precedence regardless of other modifiers.
        match key.code {
            KeyCode::Esc => {
                self.overlay.grex = None;
                return true;
            }
            KeyCode::Tab => {
                let pattern = overlay
                    .generated_pattern
                    .as_deref()
                    .filter(|p| !p.is_empty())
                    .map(str::to_string);
                if let Some(pattern) = pattern {
                    self.set_pattern(&pattern);
                    self.overlay.grex = None;
                }
                return true;
            }
            _ => {}
        }

        // Flag toggles (Alt+d/a/c). These reset the debounce so the new flags regenerate.
        if key.modifiers.contains(KeyModifiers::ALT) {
            match key.code {
                KeyCode::Char('d') => {
                    overlay.options.digit = !overlay.options.digit;
                    overlay.debounce_deadline = Some(std::time::Instant::now() + debounce);
                    return true;
                }
                KeyCode::Char('a') => {
                    overlay.options.anchors = !overlay.options.anchors;
                    overlay.debounce_deadline = Some(std::time::Instant::now() + debounce);
                    return true;
                }
                KeyCode::Char('c') => {
                    overlay.options.case_insensitive = !overlay.options.case_insensitive;
                    overlay.debounce_deadline = Some(std::time::Instant::now() + debounce);
                    return true;
                }
                _ => {}
            }
        }

        // Editor input — dispatch a focused set of keys to the overlay editor.
        let mut consumed = true;
        match key.code {
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                overlay.editor.insert_char(c);
            }
            KeyCode::Enter => overlay.editor.insert_newline(),
            KeyCode::Backspace => overlay.editor.delete_back(),
            KeyCode::Delete => overlay.editor.delete_forward(),
            KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
                overlay.editor.move_word_left();
            }
            KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
                overlay.editor.move_word_right();
            }
            KeyCode::Left => overlay.editor.move_left(),
            KeyCode::Right => overlay.editor.move_right(),
            KeyCode::Up => overlay.editor.move_up(),
            KeyCode::Down => overlay.editor.move_down(),
            KeyCode::Home => overlay.editor.move_home(),
            KeyCode::End => overlay.editor.move_end(),
            _ => consumed = false,
        }

        if consumed {
            overlay.debounce_deadline = Some(std::time::Instant::now() + debounce);
        }
        consumed
    }
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{b:02X}"));
            }
        }
    }
    out
}

fn print_colored_matches(text: &str, matches: &[engine::Match]) {
    let mut pos = 0;
    for m in matches {
        if m.start > pos {
            print!("{}", &text[pos..m.start]);
        }
        print!("{RED_BOLD}{}{RESET}", &text[m.start..m.end]);
        pos = m.end;
    }
    if pos < text.len() {
        print!("{}", &text[pos..]);
    }
    if !text.ends_with('\n') {
        println!();
    }
}

/// Print replacement output with replaced segments highlighted.
fn print_colored_replace(output: &str, segments: &[engine::ReplaceSegment]) {
    for seg in segments {
        let chunk = &output[seg.start..seg.end];
        if seg.is_replacement {
            print!("{GREEN_BOLD}{chunk}{RESET}");
        } else {
            print!("{chunk}");
        }
    }
    if !output.ends_with('\n') {
        println!();
    }
}
