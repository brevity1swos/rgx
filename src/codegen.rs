use std::fmt;

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
    pub fn all() -> &'static [Language] {
        ALL_LANGUAGES
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Rust => write!(f, "Rust"),
            Language::Python => write!(f, "Python"),
            Language::JavaScript => write!(f, "JavaScript"),
            Language::Go => write!(f, "Go"),
            Language::Java => write!(f, "Java"),
            Language::CSharp => write!(f, "C#"),
            Language::Php => write!(f, "PHP"),
            Language::Ruby => write!(f, "Ruby"),
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

/// Escape a pattern for use inside a double-quoted string literal.
fn escape_double_quoted(pattern: &str) -> String {
    pattern.replace('\\', "\\\\").replace('"', "\\\"")
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
    let escaped = escape_double_quoted(pattern);
    let has_flags = flags.case_insensitive
        || flags.multi_line
        || flags.dot_matches_newline
        || flags.unicode
        || flags.extended;

    if has_flags {
        let mut lines = String::from("use regex::RegexBuilder;\n\n");
        lines.push_str(&format!("let re = RegexBuilder::new(r\"{}\")\n", escaped));
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
             let re = Regex::new(r\"{}\").unwrap();\n\
             let matches: Vec<&str> = re.find_iter(text).map(|m| m.as_str()).collect();\n",
            escaped
        )
    }
}

fn generate_python(pattern: &str, flags: &EngineFlags) -> String {
    let escaped = escape_double_quoted(pattern);
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
             pattern = re.compile(r\"{}\")\n\
             matches = pattern.findall(text)\n",
            escaped
        )
    } else {
        format!(
            "import re\n\n\
             pattern = re.compile(r\"{}\", {})\n\
             matches = pattern.findall(text)\n",
            escaped,
            flag_parts.join(" | ")
        )
    }
}

fn generate_javascript(pattern: &str, flags: &EngineFlags) -> String {
    let escaped = pattern.replace('/', "\\/");
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
        "const regex = /{}/{js_flags};\n\
         const matches = [...text.matchAll(regex)];\n",
        escaped
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
    if flags.unicode {
        inline_flags.push('U');
    }

    let pattern_str = if inline_flags.is_empty() {
        format!("`{}`", escaped)
    } else {
        format!("`(?{}){}`", inline_flags, escaped)
    };

    format!(
        "import \"regexp\"\n\n\
         re := regexp.MustCompile({})\n\
         matches := re.FindAllString(text, -1)\n",
        pattern_str
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
             Pattern pattern = Pattern.compile(\"{}\");\n\
             Matcher matcher = pattern.matcher(text);\n\
             while (matcher.find()) {{\n\
             \x20   System.out.println(matcher.group());\n\
             }}\n",
            escaped
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
             var regex = new Regex(@\"{}\");\n\
             var matches = regex.Matches(text);\n",
            escaped
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
    let escaped = pattern.replace('\'', "\\'").replace('/', "\\/");
    let php_flags = flags.to_inline_prefix();

    format!(
        "$pattern = '/{}/{}';\n\
         preg_match_all($pattern, $text, $matches);\n",
        escaped, php_flags
    )
}

fn generate_ruby(pattern: &str, flags: &EngineFlags) -> String {
    let escaped = pattern.replace('/', "\\/");
    let mut ruby_flags = String::new();
    if flags.case_insensitive {
        ruby_flags.push('i');
    }
    if flags.multi_line {
        ruby_flags.push('m');
    }
    if flags.extended {
        ruby_flags.push('x');
    }

    format!(
        "pattern = /{}/{}\n\
         matches = text.scan(pattern)\n",
        escaped, ruby_flags
    )
}
