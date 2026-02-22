pub mod fancy;
#[cfg(feature = "pcre2-engine")]
pub mod pcre2;
pub mod rust_regex;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineKind {
    RustRegex,
    FancyRegex,
    #[cfg(feature = "pcre2-engine")]
    Pcre2,
}

impl EngineKind {
    pub fn all() -> Vec<EngineKind> {
        vec![
            EngineKind::RustRegex,
            EngineKind::FancyRegex,
            #[cfg(feature = "pcre2-engine")]
            EngineKind::Pcre2,
        ]
    }

    pub fn next(self) -> EngineKind {
        match self {
            EngineKind::RustRegex => EngineKind::FancyRegex,
            #[cfg(feature = "pcre2-engine")]
            EngineKind::FancyRegex => EngineKind::Pcre2,
            #[cfg(not(feature = "pcre2-engine"))]
            EngineKind::FancyRegex => EngineKind::RustRegex,
            #[cfg(feature = "pcre2-engine")]
            EngineKind::Pcre2 => EngineKind::RustRegex,
        }
    }
}

impl fmt::Display for EngineKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineKind::RustRegex => write!(f, "Rust regex"),
            EngineKind::FancyRegex => write!(f, "fancy-regex"),
            #[cfg(feature = "pcre2-engine")]
            EngineKind::Pcre2 => write!(f, "PCRE2"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EngineFlags {
    pub case_insensitive: bool,
    pub multi_line: bool,
    pub dot_matches_newline: bool,
    pub unicode: bool,
    pub extended: bool,
}

impl EngineFlags {
    pub fn toggle_case_insensitive(&mut self) {
        self.case_insensitive = !self.case_insensitive;
    }
    pub fn toggle_multi_line(&mut self) {
        self.multi_line = !self.multi_line;
    }
    pub fn toggle_dot_matches_newline(&mut self) {
        self.dot_matches_newline = !self.dot_matches_newline;
    }
    pub fn toggle_unicode(&mut self) {
        self.unicode = !self.unicode;
    }
    pub fn toggle_extended(&mut self) {
        self.extended = !self.extended;
    }
}

#[derive(Debug, Clone)]
pub struct Match {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub captures: Vec<CaptureGroup>,
}

#[derive(Debug, Clone)]
pub struct CaptureGroup {
    pub index: usize,
    pub name: Option<String>,
    pub start: usize,
    pub end: usize,
    pub text: String,
}

#[derive(Debug)]
pub enum EngineError {
    CompileError(String),
    MatchError(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::CompileError(msg) => write!(f, "Compile error: {msg}"),
            EngineError::MatchError(msg) => write!(f, "Match error: {msg}"),
        }
    }
}

impl std::error::Error for EngineError {}

pub type EngineResult<T> = Result<T, EngineError>;

pub trait RegexEngine: Send + Sync {
    fn kind(&self) -> EngineKind;
    fn compile(&self, pattern: &str, flags: &EngineFlags) -> EngineResult<Box<dyn CompiledRegex>>;
}

pub trait CompiledRegex: Send + Sync {
    fn find_matches(&self, text: &str) -> EngineResult<Vec<Match>>;
}

pub fn create_engine(kind: EngineKind) -> Box<dyn RegexEngine> {
    match kind {
        EngineKind::RustRegex => Box::new(rust_regex::RustRegexEngine),
        EngineKind::FancyRegex => Box::new(fancy::FancyRegexEngine),
        #[cfg(feature = "pcre2-engine")]
        EngineKind::Pcre2 => Box::new(pcre2::Pcre2Engine),
    }
}
