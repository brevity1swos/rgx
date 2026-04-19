pub mod fancy;
#[cfg(feature = "pcre2-engine")]
pub mod pcre2;
#[cfg(feature = "pcre2-engine")]
pub mod pcre2_debug;
pub mod rust_regex;

use serde::Serialize;
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

#[derive(Debug, Clone, Copy)]
pub struct EngineFlags {
    pub case_insensitive: bool,
    pub multi_line: bool,
    pub dot_matches_newline: bool,
    pub unicode: bool,
    pub extended: bool,
}

impl Default for EngineFlags {
    /// Unicode defaults to `true` to match the `regex` crate and
    /// `fancy-regex` engine behavior and the runtime `Settings` default.
    /// Using `#[derive(Default)]` would produce `unicode: false`, which
    /// (combined with `to_regex_inline_prefix` emitting `(?-u)` for
    /// byte-mode) would cause fancy-regex to reject the pattern with
    /// "Disabling Unicode not supported".
    fn default() -> Self {
        Self {
            case_insensitive: false,
            multi_line: false,
            dot_matches_newline: false,
            unicode: true,
            extended: false,
        }
    }
}

impl EngineFlags {
    /// PHP-style positive flag set — each flag enabled is a single letter.
    /// Used by the PHP codegen block where `u` enables unicode (opposite
    /// convention from the `regex` crate, which has unicode on by default).
    #[allow(clippy::wrong_self_convention)] // kept as &self for API stability; self is Copy so the borrow is free
    pub fn to_inline_prefix(&self) -> String {
        let mut s = String::new();
        if self.case_insensitive {
            s.push('i');
        }
        if self.multi_line {
            s.push('m');
        }
        if self.dot_matches_newline {
            s.push('s');
        }
        if self.unicode {
            s.push('u');
        }
        if self.extended {
            s.push('x');
        }
        s
    }

    /// `regex`-crate-style inline prefix. Unicode is **default-on** in both
    /// the `regex` crate and `fancy-regex`, so we only emit it in its
    /// disable form (`-u`) when the user has explicitly turned unicode off.
    /// Emitting `(?u)` on a pattern that also uses a fancy-only feature
    /// (lookaround / backrefs) has been observed to force fancy-regex to
    /// delegate to the non-fancy backend, which then errors — so keeping
    /// the prefix minimal is also a correctness fix, not just cleanup.
    #[allow(clippy::wrong_self_convention)] // symmetric with to_inline_prefix; self is Copy so the borrow is free
    fn to_regex_inline_prefix(&self) -> String {
        let mut enable = String::new();
        if self.case_insensitive {
            enable.push('i');
        }
        if self.multi_line {
            enable.push('m');
        }
        if self.dot_matches_newline {
            enable.push('s');
        }
        if self.extended {
            enable.push('x');
        }
        let disable_unicode = !self.unicode;
        match (enable.is_empty(), disable_unicode) {
            (true, false) => String::new(),
            (false, false) => enable,
            (true, true) => "-u".to_string(),
            (false, true) => format!("{enable}-u"),
        }
    }

    pub fn wrap_pattern(&self, pattern: &str) -> String {
        let prefix = self.to_regex_inline_prefix();
        if prefix.is_empty() {
            pattern.to_string()
        } else {
            format!("(?{prefix}){pattern}")
        }
    }

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

#[derive(Debug, Clone, Serialize)]
pub struct Match {
    #[serde(rename = "match")]
    pub text: String,
    pub start: usize,
    pub end: usize,
    #[serde(rename = "groups")]
    pub captures: Vec<CaptureGroup>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CaptureGroup {
    #[serde(rename = "group")]
    pub index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "value")]
    pub text: String,
    pub start: usize,
    pub end: usize,
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

/// Return the "power level" of an engine (higher = more capable).
fn engine_level(kind: EngineKind) -> u8 {
    match kind {
        EngineKind::RustRegex => 0,
        EngineKind::FancyRegex => 1,
        #[cfg(feature = "pcre2-engine")]
        EngineKind::Pcre2 => 2,
    }
}

/// Detect the minimum engine needed for the given pattern.
pub fn detect_minimum_engine(pattern: &str) -> EngineKind {
    #[cfg(feature = "pcre2-engine")]
    {
        if needs_pcre2(pattern) {
            return EngineKind::Pcre2;
        }
    }

    if needs_fancy(pattern) {
        return EngineKind::FancyRegex;
    }

    EngineKind::RustRegex
}

/// Return `true` if `suggested` is a strict upgrade over `current`.
pub fn is_engine_upgrade(current: EngineKind, suggested: EngineKind) -> bool {
    engine_level(suggested) > engine_level(current)
}

fn needs_fancy(pattern: &str) -> bool {
    if pattern.contains("(?=")
        || pattern.contains("(?!")
        || pattern.contains("(?<=")
        || pattern.contains("(?<!")
    {
        return true;
    }
    has_backreference(pattern)
}

fn has_backreference(pattern: &str) -> bool {
    let bytes = pattern.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len.saturating_sub(1) {
        if bytes[i] == b'\\' {
            let next = bytes[i + 1];
            if next.is_ascii_digit() && next != b'0' {
                return true;
            }
            // Skip the escaped character so we don't re-inspect it
            i += 2;
            continue;
        }
        i += 1;
    }
    false
}

#[cfg(feature = "pcre2-engine")]
fn needs_pcre2(pattern: &str) -> bool {
    if pattern.contains("(?R)")
        || pattern.contains("(*SKIP)")
        || pattern.contains("(*FAIL)")
        || pattern.contains("(*PRUNE)")
        || pattern.contains("(*COMMIT)")
        || pattern.contains("\\K")
        || pattern.contains("(?(")
    {
        return true;
    }
    has_subroutine_call(pattern)
}

#[cfg(feature = "pcre2-engine")]
fn has_subroutine_call(pattern: &str) -> bool {
    let bytes = pattern.as_bytes();
    for i in 0..bytes.len().saturating_sub(2) {
        if bytes[i] == b'('
            && bytes[i + 1] == b'?'
            && bytes.get(i + 2).is_some_and(u8::is_ascii_digit)
        {
            return true;
        }
    }
    false
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
    let mut chars = template.char_indices().peekable();

    while let Some((_i, c)) = chars.next() {
        if c == '$' {
            match chars.peek() {
                None => {
                    result.push('$');
                }
                Some(&(_, '$')) => {
                    chars.next();
                    result.push('$');
                }
                Some(&(_, '&')) => {
                    chars.next();
                    result.push_str(&m.text);
                }
                Some(&(_, '{')) => {
                    chars.next(); // consume '{'
                    let brace_start = chars.peek().map(|&(idx, _)| idx).unwrap_or(template.len());
                    if let Some(close) = template[brace_start..].find('}') {
                        let ref_name = &template[brace_start..brace_start + close];
                        if let Some(text) = lookup_capture(m, ref_name) {
                            result.push_str(text);
                        }
                        // Advance past the content and closing brace
                        let end_byte = brace_start + close + 1;
                        while chars.peek().is_some_and(|&(idx, _)| idx < end_byte) {
                            chars.next();
                        }
                    } else {
                        result.push('$');
                        result.push('{');
                    }
                }
                Some(&(_, next_c)) if next_c.is_ascii_digit() => {
                    let (_, d1) = chars.next().expect("peeked value must exist");
                    let mut num_str = String::from(d1);
                    // Grab a second digit if present
                    if let Some(&(_, d2)) = chars.peek() {
                        if d2.is_ascii_digit() {
                            chars.next();
                            num_str.push(d2);
                        }
                    }
                    let idx: usize = num_str.parse().unwrap_or(0);
                    if idx == 0 {
                        result.push_str(&m.text);
                    } else if let Some(cap) = m.captures.iter().find(|c| c.index == idx) {
                        result.push_str(&cap.text);
                    }
                }
                Some(_) => {
                    result.push('$');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Look up a capture by name or numeric string.
pub fn lookup_capture<'a>(m: &'a Match, key: &str) -> Option<&'a str> {
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
    fn test_expand_replacement_non_ascii() {
        let m = make_match(0, 5, "hello", vec![]);
        // Non-ASCII characters in replacement template should work correctly
        assert_eq!(expand_replacement("café $0", &m), "café hello");
        assert_eq!(expand_replacement("→$0←", &m), "→hello←");
        assert_eq!(expand_replacement("日本語", &m), "日本語");
        assert_eq!(expand_replacement("über $& cool", &m), "über hello cool");
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

    // --- Auto engine detection tests ---

    #[test]
    fn test_detect_simple_pattern_uses_rust_regex() {
        assert_eq!(detect_minimum_engine(r"\d+"), EngineKind::RustRegex);
        assert_eq!(detect_minimum_engine(r"[a-z]+"), EngineKind::RustRegex);
        assert_eq!(detect_minimum_engine(r"foo|bar"), EngineKind::RustRegex);
        assert_eq!(detect_minimum_engine(r"^\w+$"), EngineKind::RustRegex);
    }

    #[test]
    fn test_detect_lookahead_needs_fancy() {
        assert_eq!(detect_minimum_engine(r"foo(?=bar)"), EngineKind::FancyRegex);
        assert_eq!(detect_minimum_engine(r"foo(?!bar)"), EngineKind::FancyRegex);
    }

    #[test]
    fn test_detect_lookbehind_needs_fancy() {
        assert_eq!(
            detect_minimum_engine(r"(?<=foo)bar"),
            EngineKind::FancyRegex,
        );
        assert_eq!(
            detect_minimum_engine(r"(?<!foo)bar"),
            EngineKind::FancyRegex,
        );
    }

    #[test]
    fn test_detect_backreference_needs_fancy() {
        assert_eq!(detect_minimum_engine(r"(\w+)\s+\1"), EngineKind::FancyRegex,);
        assert_eq!(detect_minimum_engine(r"(a)(b)\2"), EngineKind::FancyRegex);
    }

    #[test]
    fn test_detect_non_backreference_escapes_stay_rust() {
        // These look like \digit but are actually common escapes
        assert_eq!(detect_minimum_engine(r"\d"), EngineKind::RustRegex);
        assert_eq!(detect_minimum_engine(r"\w\s\b"), EngineKind::RustRegex);
        assert_eq!(detect_minimum_engine(r"\0"), EngineKind::RustRegex);
        assert_eq!(detect_minimum_engine(r"\n\r\t"), EngineKind::RustRegex);
        assert_eq!(detect_minimum_engine(r"\x41"), EngineKind::RustRegex);
        assert_eq!(detect_minimum_engine(r"\u0041"), EngineKind::RustRegex);
        assert_eq!(detect_minimum_engine(r"\p{L}"), EngineKind::RustRegex);
        assert_eq!(detect_minimum_engine(r"\P{L}"), EngineKind::RustRegex);
        assert_eq!(detect_minimum_engine(r"\B"), EngineKind::RustRegex);
    }

    #[test]
    fn test_has_backreference() {
        assert!(has_backreference(r"(\w+)\1"));
        assert!(has_backreference(r"\1"));
        assert!(has_backreference(r"(a)(b)(c)\3"));
        assert!(!has_backreference(r"\d+"));
        assert!(!has_backreference(r"\0"));
        assert!(!has_backreference(r"plain text"));
        assert!(!has_backreference(r"\w\s\b\B\n\r\t"));
    }

    #[test]
    fn test_detect_empty_pattern() {
        assert_eq!(detect_minimum_engine(""), EngineKind::RustRegex);
    }

    #[test]
    fn test_is_engine_upgrade() {
        assert!(is_engine_upgrade(
            EngineKind::RustRegex,
            EngineKind::FancyRegex
        ));
        assert!(!is_engine_upgrade(
            EngineKind::FancyRegex,
            EngineKind::RustRegex
        ));
        assert!(!is_engine_upgrade(
            EngineKind::FancyRegex,
            EngineKind::FancyRegex,
        ));
    }

    #[test]
    fn wrap_pattern_omits_prefix_when_flags_are_defaults() {
        // All flags at default (unicode on, everything else off) → no prefix.
        let flags = EngineFlags::default();
        assert_eq!(flags.wrap_pattern("abc"), "abc");
    }

    #[test]
    fn wrap_pattern_emits_minus_u_when_unicode_disabled() {
        let flags = EngineFlags {
            unicode: false,
            ..EngineFlags::default()
        };
        assert_eq!(flags.wrap_pattern("abc"), "(?-u)abc");
    }

    #[test]
    fn wrap_pattern_combines_enable_and_disable_unicode() {
        let flags = EngineFlags {
            case_insensitive: true,
            unicode: false,
            ..EngineFlags::default()
        };
        assert_eq!(flags.wrap_pattern("abc"), "(?i-u)abc");
    }

    #[test]
    fn wrap_pattern_does_not_emit_u_when_unicode_on() {
        // Regression guard: emitting `(?u)` trips fancy-regex's backend
        // routing on lookaround patterns in our build. Unicode being
        // on-by-default means the prefix adds nothing but risk.
        let flags = EngineFlags {
            case_insensitive: true,
            unicode: true,
            ..EngineFlags::default()
        };
        assert_eq!(flags.wrap_pattern("abc"), "(?i)abc");
    }

    #[test]
    fn to_inline_prefix_still_emits_positive_u_for_php() {
        // The PHP codegen uses `to_inline_prefix` — PHP's `/pattern/u`
        // delimiter enables unicode, opposite of the regex-crate
        // convention. This test locks in the split between the two
        // prefix methods.
        let flags = EngineFlags {
            case_insensitive: true,
            unicode: true,
            ..EngineFlags::default()
        };
        assert_eq!(flags.to_inline_prefix(), "iu");
    }

    #[cfg(feature = "pcre2-engine")]
    mod pcre2_detection_tests {
        use super::*;

        #[test]
        fn test_detect_recursion_needs_pcre2() {
            assert_eq!(detect_minimum_engine(r"(?R)"), EngineKind::Pcre2);
        }

        #[test]
        fn test_detect_backtracking_verbs_need_pcre2() {
            assert_eq!(detect_minimum_engine(r"(*SKIP)(*FAIL)"), EngineKind::Pcre2);
            assert_eq!(detect_minimum_engine(r"(*PRUNE)"), EngineKind::Pcre2);
            assert_eq!(detect_minimum_engine(r"(*COMMIT)"), EngineKind::Pcre2);
        }

        #[test]
        fn test_detect_reset_match_start_needs_pcre2() {
            assert_eq!(detect_minimum_engine(r"foo\Kbar"), EngineKind::Pcre2);
        }

        #[test]
        fn test_detect_conditional_needs_pcre2() {
            assert_eq!(detect_minimum_engine(r"(?(1)yes|no)"), EngineKind::Pcre2,);
        }

        #[test]
        fn test_detect_subroutine_call_needs_pcre2() {
            assert_eq!(detect_minimum_engine(r"(\d+)(?1)"), EngineKind::Pcre2);
        }

        #[test]
        fn test_is_engine_upgrade_pcre2() {
            assert!(is_engine_upgrade(EngineKind::RustRegex, EngineKind::Pcre2));
            assert!(is_engine_upgrade(EngineKind::FancyRegex, EngineKind::Pcre2));
            assert!(!is_engine_upgrade(
                EngineKind::Pcre2,
                EngineKind::FancyRegex
            ));
            assert!(!is_engine_upgrade(EngineKind::Pcre2, EngineKind::RustRegex));
        }
    }
}
