use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::engine::{self, CompiledRegex, EngineFlags, EngineKind, RegexEngine};
use crate::explain::{self, ExplainNode};
use crate::input::editor::Editor;

// ANSI escape codes for batch output coloring
const ANSI_RED_BOLD: &str = "\x1b[1;31m";
const ANSI_GREEN_BOLD: &str = "\x1b[1;32m";
const ANSI_RESET: &str = "\x1b[0m";

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
    pub show_help: bool,
    pub help_page: usize,
    pub should_quit: bool,
    pub match_scroll: u16,
    pub replace_scroll: u16,
    pub explain_scroll: u16,
    // Pattern history
    pub pattern_history: VecDeque<String>,
    pub history_index: Option<usize>,
    history_temp: Option<String>,
    // Match selection + clipboard
    pub selected_match: usize,
    pub selected_capture: Option<usize>,
    pub clipboard_status: Option<String>,
    clipboard_status_ticks: u32,
    pub show_whitespace: bool,
    pub rounded_borders: bool,
    pub vim_mode: bool,
    pub vim_state: crate::input::vim::VimState,
    pub compile_time: Option<Duration>,
    pub match_time: Option<Duration>,
    pub error_offset: Option<usize>,
    pub output_on_quit: bool,
    pub workspace_path: Option<String>,
    pub show_recipes: bool,
    pub recipe_index: usize,
    pub show_benchmark: bool,
    pub benchmark_results: Vec<BenchmarkResult>,
    pub show_codegen: bool,
    pub codegen_language_index: usize,
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
            show_help: false,
            help_page: 0,
            should_quit: false,
            match_scroll: 0,
            replace_scroll: 0,
            explain_scroll: 0,
            pattern_history: VecDeque::new(),
            history_index: None,
            history_temp: None,
            selected_match: 0,
            selected_capture: None,
            clipboard_status: None,
            clipboard_status_ticks: 0,
            show_whitespace: false,
            rounded_borders: false,
            vim_mode: false,
            vim_state: crate::input::vim::VimState::new(),
            compile_time: None,
            match_time: None,
            error_offset: None,
            output_on_quit: false,
            workspace_path: None,
            show_recipes: false,
            recipe_index: 0,
            show_benchmark: false,
            benchmark_results: Vec::new(),
            show_codegen: false,
            codegen_language_index: 0,
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

    pub fn switch_engine_to(&mut self, kind: EngineKind) {
        self.engine_kind = kind;
        self.engine = engine::create_engine(kind);
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
        self.error_offset = None;

        if pattern.is_empty() {
            self.compiled = None;
            self.matches.clear();
            self.explanation.clear();
            self.error = None;
            self.compile_time = None;
            self.match_time = None;
            return;
        }

        // Auto-select engine: upgrade (never downgrade) if the pattern
        // requires a more powerful engine than the currently active one.
        let suggested = engine::detect_minimum_engine(&pattern);
        if engine::is_engine_upgrade(self.engine_kind, suggested) {
            let prev = self.engine_kind;
            self.engine_kind = suggested;
            self.engine = engine::create_engine(suggested);
            self.set_status_message(format!(
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
        self.match_scroll = 0;
        self.selected_match = 0;
        self.selected_capture = None;
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
        if self.pattern_history.back().map(|s| s.as_str()) == Some(&pattern) {
            return;
        }
        self.pattern_history.push_back(pattern);
        if self.pattern_history.len() > 100 {
            self.pattern_history.pop_front();
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
        if self.matches.is_empty() || self.selected_match >= self.matches.len() {
            return;
        }
        let mut line = 0usize;
        for i in 0..self.selected_match {
            line += 1 + self.matches[i].captures.len();
        }
        if let Some(ci) = self.selected_capture {
            line += 1 + ci;
        }
        self.match_scroll = u16::try_from(line).unwrap_or(u16::MAX);
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
                Ok(()) => self.set_status_message(success_msg.to_string()),
                Err(e) => self.set_status_message(format!("Clipboard error: {e}")),
            },
            Err(e) => self.set_status_message(format!("Clipboard error: {e}")),
        }
    }

    pub fn set_status_message(&mut self, message: String) {
        self.clipboard_status = Some(message);
        self.clipboard_status_ticks = 40; // ~2 sec at 50ms tick
    }

    /// Tick down the clipboard status timer. Returns true if status was cleared.
    pub fn tick_clipboard_status(&mut self) -> bool {
        if self.clipboard_status.is_some() {
            if self.clipboard_status_ticks > 0 {
                self.clipboard_status_ticks -= 1;
            } else {
                self.clipboard_status = None;
                return true;
            }
        }
        false
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
                        println!("{ANSI_RED_BOLD}{text}{ANSI_RESET}");
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
        let m = self.matches.get(self.selected_match)?;
        match self.selected_capture {
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
        self.show_benchmark = true;
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
            self.set_status_message("No pattern to generate code for".to_string());
            return;
        }
        let code = crate::codegen::generate_code(lang, &pattern, &self.flags);
        self.copy_to_clipboard(&code, &format!("{} code copied to clipboard", lang));
        self.show_codegen = false;
    }

    #[cfg(feature = "pcre2-engine")]
    pub fn start_debug(&mut self, max_steps: usize) {
        use crate::engine::pcre2_debug::{self, DebugSession};

        let pattern = self.regex_editor.content().to_string();
        let subject = self.test_editor.content().to_string();
        if pattern.is_empty() || subject.is_empty() {
            self.set_status_message("Debugger needs both a pattern and test string".to_string());
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
                self.set_status_message(format!("Debugger error: {e}"));
            }
        }
    }

    #[cfg(not(feature = "pcre2-engine"))]
    pub fn start_debug(&mut self, _max_steps: usize) {
        self.set_status_message(
            "Debugger requires PCRE2 (build with --features pcre2-engine)".to_string(),
        );
    }

    #[cfg(feature = "pcre2-engine")]
    fn selected_match_start(&self) -> usize {
        if !self.matches.is_empty() && self.selected_match < self.matches.len() {
            self.matches[self.selected_match].start
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
}

/// Print the test string with matches highlighted using ANSI colors.
fn print_colored_matches(text: &str, matches: &[engine::Match]) {
    let mut pos = 0;
    for m in matches {
        if m.start > pos {
            print!("{}", &text[pos..m.start]);
        }
        print!("{ANSI_RED_BOLD}{}{ANSI_RESET}", &text[m.start..m.end]);
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
            print!("{ANSI_GREEN_BOLD}{chunk}{ANSI_RESET}");
        } else {
            print!("{chunk}");
        }
    }
    if !output.ends_with('\n') {
        println!();
    }
}
