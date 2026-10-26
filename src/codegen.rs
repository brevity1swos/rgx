use std::fmt;
use std::fmt::Write as _;

use crate::engine::EngineFlags;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    Go,
    Java,
    CSharp,
    Php,
    Ruby,
}

pub const ALL_LANGUAGES: &[Language] = &[
    Language::Rust,
    Language::Python,
    Language::JavaScript,
    Language::Go,
    Language::Java,
    Language::CSharp,
    Language::Php,
    Language::Ruby,
];

impl Language {
    pub const fn all() -> &'static [Self] {
        ALL_LANGUAGES
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rust => write!(f, "Rust"),
            Self::Python => write!(f, "Python"),
            Self::JavaScript => write!(f, "JavaScript"),
            Self::Go => write!(f, "Go"),
            Self::Java => write!(f, "Java"),
            Self::CSharp => write!(f, "C#"),
            Self::Php => write!(f, "PHP"),
            Self::Ruby => write!(f, "Ruby"),
        }
    }
}

pub fn generate_code(lang: &Language, pattern: &str, flags: &EngineFlags) -> String {
    match lang {
        Language::Rust => generate_rust(pattern, flags),
        Language::Python => generate_python(pattern, flags),
        Language::JavaScript => generate_javascript(pattern, flags),
        Language::Go => generate_go(pattern, flags),
        Language::Java => generate_java(pattern, flags),
        Language::CSharp => generate_csharp(pattern, flags),
        Language::Php => generate_php(pattern, flags),
        Language::Ruby => generate_ruby(pattern, flags),
    }
}

/// Escape a pattern for use inside a (non-raw) double-quoted string literal.
fn escape_double_quoted(pattern: &str) -> String {
    let mut out = String::with_capacity(pattern.len());
    for c in pattern.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

/// Rust raw string literal with enough `#`s that the pattern cannot close it.
fn rust_raw_string(pattern: &str) -> String {
    let mut hashes = 0;
    while pattern.contains(&format!("\"{}", "#".repeat(hashes))) {
        hashes += 1;
    }
    let h = "#".repeat(hashes);
    format!("r{h}\"{pattern}\"{h}")
}

/// Python string literal: raw when a quote style allows it, escaped otherwise.
fn python_string(pattern: &str) -> String {
    if !pattern.contains('\n') && !pattern.contains('\r') {
        if !pattern.contains('"') {
            return format!("r\"{pattern}\"");
        }
        if !pattern.contains('\'') {
            return format!("r'{pattern}'");
        }
    }
    format!("\"{}\"", escape_double_quoted(pattern))
}

/// Host syntax of a `/.../` regex literal.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SlashLiteral {
    /// Raw line breaks are illegal inside a JS regex literal.
    JavaScript,
    /// Ruby interpolates `#{}`, `#@var` and `#$var` inside regex literals.
    Ruby,
    /// The PHP delimiter sits inside a single-quoted string (escaped separately).
    Php,
}

/// Escape `/` for a `/.../` regex literal, leaving already-escaped chars intact,
/// plus whatever else `host` would otherwise interpret.
fn escape_slash_delimiter(pattern: &str, host: SlashLiteral) -> String {
    let mut out = String::with_capacity(pattern.len());
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                out.push('\\');
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            }
            '/' => out.push_str("\\/"),
            '\n' if host == SlashLiteral::JavaScript => out.push_str("\\n"),
            '\r' if host == SlashLiteral::JavaScript => out.push_str("\\r"),
            // `\#` is a literal `#` in a Ruby regex and blocks interpolation.
            '#' if host == SlashLiteral::Ruby && matches!(chars.peek(), Some('{' | '@' | '$')) => {
                out.push_str("\\#");
            }
            _ => out.push(c),
        }
    }
    out
}

/// Collect active flag names from a language-specific mapping.
fn collect_flags<'a>(mapping: &[(&'a str, bool)]) -> Vec<&'a str> {
    mapping
        .iter()
        .filter(|(_, active)| *active)
        .map(|(name, _)| *name)
        .collect()
}

fn generate_rust(pattern: &str, flags: &EngineFlags) -> String {
    let literal = rust_raw_string(pattern);
    let has_flags = flags.case_insensitive
        || flags.multi_line
        || flags.dot_matches_newline
        || flags.unicode
        || flags.extended;

    if has_flags {
        let mut lines = String::from("use regex::RegexBuilder;\n\n");
        let _ = writeln!(lines, "let re = RegexBuilder::new({literal})");
        if flags.case_insensitive {
            lines.push_str("    .case_insensitive(true)\n");
        }
        if flags.multi_line {
            lines.push_str("    .multi_line(true)\n");
        }
        if flags.dot_matches_newline {
            lines.push_str("    .dot_matches_new_line(true)\n");
        }
        if flags.unicode {
            lines.push_str("    .unicode(true)\n");
        }
        if flags.extended {
            lines.push_str("    .ignore_whitespace(true)\n");
        }
        lines.push_str("    .build()\n    .unwrap();\n");
        lines.push_str(
            "let matches: Vec<&str> = re.find_iter(text).map(|m| m.as_str()).collect();\n",
        );
        lines
    } else {
        format!(
            "use regex::Regex;\n\n\
             let re = Regex::new({literal}).unwrap();\n\
             let matches: Vec<&str> = re.find_iter(text).map(|m| m.as_str()).collect();\n"
        )
    }
}

fn generate_python(pattern: &str, flags: &EngineFlags) -> String {
    let literal = python_string(pattern);
    let flag_parts = collect_flags(&[
        ("re.IGNORECASE", flags.case_insensitive),
        ("re.MULTILINE", flags.multi_line),
        ("re.DOTALL", flags.dot_matches_newline),
        ("re.UNICODE", flags.unicode),
        ("re.VERBOSE", flags.extended),
    ]);

    if flag_parts.is_empty() {
        format!(
            "import re\n\n\
             pattern = re.compile({literal})\n\
             matches = pattern.findall(text)\n"
        )
    } else {
        format!(
            "import re\n\n\
             pattern = re.compile({}, {})\n\
             matches = pattern.findall(text)\n",
            literal,
            flag_parts.join(" | ")
        )
    }
}

fn generate_javascript(pattern: &str, flags: &EngineFlags) -> String {
    let escaped = escape_slash_delimiter(pattern, SlashLiteral::JavaScript);
    let mut js_flags = String::from("g");
    if flags.case_insensitive {
        js_flags.push('i');
    }
    if flags.multi_line {
        js_flags.push('m');
    }
    if flags.dot_matches_newline {
        js_flags.push('s');
    }
    if flags.unicode {
        js_flags.push('u');
    }

    format!(
        "const regex = /{escaped}/{js_flags};\n\
         const matches = [...text.matchAll(regex)];\n"
    )
}

fn generate_go(pattern: &str, flags: &EngineFlags) -> String {
    let escaped = pattern.replace('`', "`+\"`\"+`");
    let mut inline_flags = String::new();
    if flags.case_insensitive {
        inline_flags.push('i');
    }
    if flags.multi_line {
        inline_flags.push('m');
    }
    if flags.dot_matches_newline {
        inline_flags.push('s');
    }
    // Go regexps are always Unicode-aware; `(?U)` would mean "ungreedy".

    let pattern_str = if inline_flags.is_empty() {
        format!("`{escaped}`")
    } else {
        format!("`(?{inline_flags}){escaped}`")
    };

    format!(
        "import \"regexp\"\n\n\
         re := regexp.MustCompile({pattern_str})\n\
         matches := re.FindAllString(text, -1)\n"
    )
}

fn generate_java(pattern: &str, flags: &EngineFlags) -> String {
    let escaped = escape_double_quoted(pattern);
    let flag_parts = collect_flags(&[
        ("Pattern.CASE_INSENSITIVE", flags.case_insensitive),
        ("Pattern.MULTILINE", flags.multi_line),
        ("Pattern.DOTALL", flags.dot_matches_newline),
        ("Pattern.UNICODE_CHARACTER_CLASS", flags.unicode),
        ("Pattern.COMMENTS", flags.extended),
    ]);

    if flag_parts.is_empty() {
        format!(
            "import java.util.regex.*;\n\n\
             Pattern pattern = Pattern.compile(\"{escaped}\");\n\
             Matcher matcher = pattern.matcher(text);\n\
             while (matcher.find()) {{\n\
             \x20   System.out.println(matcher.group());\n\
             }}\n"
        )
    } else {
        format!(
            "import java.util.regex.*;\n\n\
             Pattern pattern = Pattern.compile(\"{}\", {});\n\
             Matcher matcher = pattern.matcher(text);\n\
             while (matcher.find()) {{\n\
             \x20   System.out.println(matcher.group());\n\
             }}\n",
            escaped,
            flag_parts.join(" | ")
        )
    }
}

fn generate_csharp(pattern: &str, flags: &EngineFlags) -> String {
    let escaped = pattern.replace('"', "\"\"");
    let flag_parts = collect_flags(&[
        ("RegexOptions.IgnoreCase", flags.case_insensitive),
        ("RegexOptions.Multiline", flags.multi_line),
        ("RegexOptions.Singleline", flags.dot_matches_newline),
        ("RegexOptions.IgnorePatternWhitespace", flags.extended),
    ]);

    if flag_parts.is_empty() {
        format!(
            "using System.Text.RegularExpressions;\n\n\
             var regex = new Regex(@\"{escaped}\");\n\
             var matches = regex.Matches(text);\n"
        )
    } else {
        format!(
            "using System.Text.RegularExpressions;\n\n\
             var regex = new Regex(@\"{}\", {});\n\
             var matches = regex.Matches(text);\n",
            escaped,
            flag_parts.join(" | ")
        )
    }
}

fn generate_php(pattern: &str, flags: &EngineFlags) -> String {
    // Single-quoted PHP string: only `\\` and `\'` are escapes, so double every
    // backslash to keep the pattern byte-for-byte.
    let escaped = escape_slash_delimiter(pattern, SlashLiteral::Php)
        .replace('\\', "\\\\")
        .replace('\'', "\\'");
    let php_flags = flags.to_inline_prefix();

    format!(
        "$pattern = '/{escaped}/{php_flags}';\n\
         preg_match_all($pattern, $text, $matches);\n"
    )
}

fn generate_ruby(pattern: &str, flags: &EngineFlags) -> String {
    let escaped = escape_slash_delimiter(pattern, SlashLiteral::Ruby);
    let mut ruby_flags = String::new();
    if flags.case_insensitive {
        ruby_flags.push('i');
    }
    // Ruby's `^`/`$` always match at line breaks; its `m` flag means dot-all.
    if flags.dot_matches_newline {
        ruby_flags.push('m');
    }
    if flags.extended {
        ruby_flags.push('x');
    }

    format!(
        "pattern = /{escaped}/{ruby_flags}\n\
         matches = text.scan(pattern)\n"
    )
}
