use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::app::App;
use crate::engine::{self, EngineFlags, EngineKind};

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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tests: Vec<TestCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub input: String,
    pub should_match: bool,
}

#[derive(Debug)]
pub struct TestResult {
    pub input: String,
    pub should_match: bool,
    pub did_match: bool,
    pub passed: bool,
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
            tests: Vec::new(),
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

    /// Run test assertions and return results.
    pub fn run_tests(&self) -> anyhow::Result<Vec<TestResult>> {
        let engine_kind = match self.engine.as_str() {
            "fancy" => EngineKind::FancyRegex,
            #[cfg(feature = "pcre2-engine")]
            "pcre2" => EngineKind::Pcre2,
            _ => EngineKind::RustRegex,
        };
        let flags = EngineFlags {
            case_insensitive: self.case_insensitive,
            multi_line: self.multiline,
            dot_matches_newline: self.dotall,
            unicode: self.unicode,
            extended: self.extended,
        };
        let eng = engine::create_engine(engine_kind);
        let compiled = eng
            .compile(&self.pattern, &flags)
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let mut results = Vec::with_capacity(self.tests.len());
        for tc in &self.tests {
            let did_match = match compiled.find_matches(&tc.input) {
                Ok(m) => !m.is_empty(),
                Err(_) => false,
            };
            results.push(TestResult {
                input: tc.input.clone(),
                should_match: tc.should_match,
                did_match,
                passed: did_match == tc.should_match,
            });
        }
        Ok(results)
    }
}

const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_RED: &str = "\x1b[31m";
const ANSI_BOLD: &str = "\x1b[1m";
const ANSI_RESET: &str = "\x1b[0m";

/// Print test results to stdout. Returns true if all passed.
pub fn print_test_results(path: &str, pattern: &str, results: &[TestResult], color: bool) -> bool {
    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;

    if color {
        println!("{ANSI_BOLD}Testing:{ANSI_RESET} {path}");
        println!("{ANSI_BOLD}Pattern:{ANSI_RESET} {pattern}");
    } else {
        println!("Testing: {path}");
        println!("Pattern: {pattern}");
    }
    println!();

    for (i, r) in results.iter().enumerate() {
        let status = if r.passed {
            if color {
                format!("{ANSI_GREEN}PASS{ANSI_RESET}")
            } else {
                "PASS".to_string()
            }
        } else if color {
            format!("{ANSI_RED}FAIL{ANSI_RESET}")
        } else {
            "FAIL".to_string()
        };
        let expect = if r.should_match { "match" } else { "no match" };
        let got = if r.did_match { "matched" } else { "no match" };
        println!(
            "  {status} [{:>2}] {:?} (expect {expect}, got {got})",
            i + 1,
            r.input
        );
    }

    println!();
    if failed == 0 {
        if color {
            println!("{ANSI_GREEN}{ANSI_BOLD}{passed}/{total} passed{ANSI_RESET}");
        } else {
            println!("{passed}/{total} passed");
        }
    } else if color {
        println!("{ANSI_RED}{ANSI_BOLD}{failed}/{total} failed{ANSI_RESET}");
    } else {
        println!("{failed}/{total} failed");
    }

    failed == 0
}
