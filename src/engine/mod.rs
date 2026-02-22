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

// --- Replace/Substitution support ---

#[derive(Debug, Clone)]
pub struct ReplaceSegment {
    pub start: usize,
    pub end: usize,
    pub is_replacement: bool,
}

#[derive(Debug, Clone)]
pub struct ReplaceResult {
    pub output: String,
    pub segments: Vec<ReplaceSegment>,
}

/// Expand a replacement template against a single match.
///
/// Supports: `$0` / `$&` (whole match), `$1`..`$99` (numbered groups),
/// `${name}` (named groups), `$$` (literal `$`).
fn expand_replacement(template: &str, m: &Match) -> String {
    let mut result = String::new();
    let bytes = template.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'$' {
            if i + 1 >= len {
                result.push('$');
                i += 1;
                continue;
            }
            let next = bytes[i + 1];
            if next == b'$' {
                // Literal $
                result.push('$');
                i += 2;
            } else if next == b'&' {
                // $& = whole match ($0)
                result.push_str(&m.text);
                i += 2;
            } else if next == b'{' {
                // ${name} or ${number}
                if let Some(close) = template[i + 2..].find('}') {
                    let ref_name = &template[i + 2..i + 2 + close];
                    if let Some(text) = lookup_capture(m, ref_name) {
                        result.push_str(text);
                    }
                    i = i + 2 + close + 1;
                } else {
                    // No closing brace, emit literal
                    result.push('$');
                    i += 1;
                }
            } else if next.is_ascii_digit() {
                // $1..$99
                let start = i + 1;
                let mut end = start + 1;
                // Grab up to 2 digits
                if end < len && bytes[end].is_ascii_digit() {
                    end += 1;
                }
                let num_str = &template[start..end];
                let idx: usize = num_str.parse().unwrap_or(0);
                if idx == 0 {
                    result.push_str(&m.text);
                } else if let Some(cap) = m.captures.iter().find(|c| c.index == idx) {
                    result.push_str(&cap.text);
                }
                i = end;
            } else {
                result.push('$');
                i += 1;
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

/// Look up a capture by name or numeric string.
fn lookup_capture<'a>(m: &'a Match, key: &str) -> Option<&'a str> {
    // Try as number first
    if let Ok(idx) = key.parse::<usize>() {
        if idx == 0 {
            return Some(&m.text);
        }
        return m
            .captures
            .iter()
            .find(|c| c.index == idx)
            .map(|c| c.text.as_str());
    }
    // Try as named capture
    m.captures
        .iter()
        .find(|c| c.name.as_deref() == Some(key))
        .map(|c| c.text.as_str())
}

/// Perform replacement across all matches, returning the output string and segment metadata.
pub fn replace_all(text: &str, matches: &[Match], template: &str) -> ReplaceResult {
    let mut output = String::new();
    let mut segments = Vec::new();
    let mut pos = 0;

    for m in matches {
        // Original text before this match
        if m.start > pos {
            let seg_start = output.len();
            output.push_str(&text[pos..m.start]);
            segments.push(ReplaceSegment {
                start: seg_start,
                end: output.len(),
                is_replacement: false,
            });
        }
        // Expanded replacement
        let expanded = expand_replacement(template, m);
        if !expanded.is_empty() {
            let seg_start = output.len();
            output.push_str(&expanded);
            segments.push(ReplaceSegment {
                start: seg_start,
                end: output.len(),
                is_replacement: true,
            });
        }
        pos = m.end;
    }

    // Trailing original text
    if pos < text.len() {
        let seg_start = output.len();
        output.push_str(&text[pos..]);
        segments.push(ReplaceSegment {
            start: seg_start,
            end: output.len(),
            is_replacement: false,
        });
    }

    ReplaceResult { output, segments }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_match(start: usize, end: usize, text: &str, captures: Vec<CaptureGroup>) -> Match {
        Match {
            start,
            end,
            text: text.to_string(),
            captures,
        }
    }

    fn make_cap(
        index: usize,
        name: Option<&str>,
        start: usize,
        end: usize,
        text: &str,
    ) -> CaptureGroup {
        CaptureGroup {
            index,
            name: name.map(|s| s.to_string()),
            start,
            end,
            text: text.to_string(),
        }
    }

    #[test]
    fn test_replace_all_basic() {
        let matches = vec![make_match(
            0,
            12,
            "user@example",
            vec![
                make_cap(1, None, 0, 4, "user"),
                make_cap(2, None, 5, 12, "example"),
            ],
        )];
        let result = replace_all("user@example", &matches, "$2=$1");
        assert_eq!(result.output, "example=user");
    }

    #[test]
    fn test_replace_all_no_matches() {
        let result = replace_all("hello world", &[], "replacement");
        assert_eq!(result.output, "hello world");
        assert_eq!(result.segments.len(), 1);
        assert!(!result.segments[0].is_replacement);
    }

    #[test]
    fn test_replace_all_empty_template() {
        let matches = vec![
            make_match(4, 7, "123", vec![]),
            make_match(12, 15, "456", vec![]),
        ];
        let result = replace_all("abc 123 def 456 ghi", &matches, "");
        assert_eq!(result.output, "abc  def  ghi");
    }

    #[test]
    fn test_replace_all_literal_dollar() {
        let matches = vec![make_match(0, 3, "foo", vec![])];
        let result = replace_all("foo", &matches, "$$bar");
        assert_eq!(result.output, "$bar");
    }

    #[test]
    fn test_replace_all_named_groups() {
        let matches = vec![make_match(
            0,
            7,
            "2024-01",
            vec![
                make_cap(1, Some("y"), 0, 4, "2024"),
                make_cap(2, Some("m"), 5, 7, "01"),
            ],
        )];
        let result = replace_all("2024-01", &matches, "${m}/${y}");
        assert_eq!(result.output, "01/2024");
    }

    #[test]
    fn test_expand_replacement_whole_match() {
        let m = make_match(0, 5, "hello", vec![]);
        assert_eq!(expand_replacement("$0", &m), "hello");
        assert_eq!(expand_replacement("$&", &m), "hello");
        assert_eq!(expand_replacement("[$0]", &m), "[hello]");
    }

    #[test]
    fn test_replace_segments_tracking() {
        let matches = vec![make_match(6, 9, "123", vec![])];
        let result = replace_all("hello 123 world", &matches, "NUM");
        assert_eq!(result.output, "hello NUM world");
        assert_eq!(result.segments.len(), 3);
        // "hello " - original
        assert!(!result.segments[0].is_replacement);
        assert_eq!(
            &result.output[result.segments[0].start..result.segments[0].end],
            "hello "
        );
        // "NUM" - replacement
        assert!(result.segments[1].is_replacement);
        assert_eq!(
            &result.output[result.segments[1].start..result.segments[1].end],
            "NUM"
        );
        // " world" - original
        assert!(!result.segments[2].is_replacement);
        assert_eq!(
            &result.output[result.segments[2].start..result.segments[2].end],
            " world"
        );
    }
}
