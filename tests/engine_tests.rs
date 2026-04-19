use rgx::engine::{create_engine, EngineFlags, EngineKind};

fn test_engine_basic_matching(kind: EngineKind) {
    let engine = create_engine(kind);
    let flags = EngineFlags::default();
    let compiled = engine.compile(r"\d+", &flags).unwrap();
    let matches = compiled.find_matches("hello 123 world 456").unwrap();
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].text, "123");
    assert_eq!(matches[0].start, 6);
    assert_eq!(matches[0].end, 9);
    assert_eq!(matches[1].text, "456");
}

fn test_engine_capture_groups(kind: EngineKind) {
    let engine = create_engine(kind);
    let flags = EngineFlags::default();
    let compiled = engine.compile(r"(\w+)@(\w+)", &flags).unwrap();
    let matches = compiled.find_matches("user@example").unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].captures.len(), 2);
    assert_eq!(matches[0].captures[0].text, "user");
    assert_eq!(matches[0].captures[1].text, "example");
}

fn test_engine_no_match(kind: EngineKind) {
    let engine = create_engine(kind);
    let flags = EngineFlags::default();
    let compiled = engine.compile(r"\d+", &flags).unwrap();
    let matches = compiled.find_matches("no digits here").unwrap();
    assert_eq!(matches.len(), 0);
}

fn test_engine_case_insensitive(kind: EngineKind) {
    let engine = create_engine(kind);
    let flags = EngineFlags {
        case_insensitive: true,
        ..Default::default()
    };
    let compiled = engine.compile(r"hello", &flags).unwrap();
    let matches = compiled.find_matches("Hello HELLO hello").unwrap();
    assert_eq!(matches.len(), 3);
}

fn test_engine_named_captures(kind: EngineKind) {
    let engine = create_engine(kind);
    let flags = EngineFlags::default();
    let compiled = engine
        .compile(r"(?P<user>\w+)@(?P<domain>\w+)", &flags)
        .unwrap();
    let matches = compiled.find_matches("user@example").unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].captures.len(), 2);
    assert_eq!(matches[0].captures[0].name, Some("user".to_string()));
    assert_eq!(matches[0].captures[0].text, "user");
    assert_eq!(matches[0].captures[1].name, Some("domain".to_string()));
    assert_eq!(matches[0].captures[1].text, "example");
}

fn test_engine_compile_error(kind: EngineKind) {
    let engine = create_engine(kind);
    let flags = EngineFlags::default();
    assert!(engine.compile(r"(unclosed", &flags).is_err());
}

#[test]
fn rust_regex_basic() {
    test_engine_basic_matching(EngineKind::RustRegex);
}

#[test]
fn rust_regex_captures() {
    test_engine_capture_groups(EngineKind::RustRegex);
}

#[test]
fn rust_regex_no_match() {
    test_engine_no_match(EngineKind::RustRegex);
}

#[test]
fn rust_regex_case_insensitive() {
    test_engine_case_insensitive(EngineKind::RustRegex);
}

#[test]
fn rust_regex_named_captures() {
    test_engine_named_captures(EngineKind::RustRegex);
}

#[test]
fn rust_regex_compile_error() {
    test_engine_compile_error(EngineKind::RustRegex);
}

#[test]
fn fancy_regex_basic() {
    test_engine_basic_matching(EngineKind::FancyRegex);
}

#[test]
fn fancy_regex_captures() {
    test_engine_capture_groups(EngineKind::FancyRegex);
}

#[test]
fn fancy_regex_no_match() {
    test_engine_no_match(EngineKind::FancyRegex);
}

#[test]
fn fancy_regex_case_insensitive() {
    test_engine_case_insensitive(EngineKind::FancyRegex);
}

#[test]
fn fancy_regex_named_captures() {
    test_engine_named_captures(EngineKind::FancyRegex);
}

#[test]
fn fancy_regex_compile_error() {
    test_engine_compile_error(EngineKind::FancyRegex);
}

#[cfg(feature = "pcre2-engine")]
mod pcre2_tests {
    use super::*;

    #[test]
    fn pcre2_basic() {
        test_engine_basic_matching(EngineKind::Pcre2);
    }

    #[test]
    fn pcre2_captures() {
        test_engine_capture_groups(EngineKind::Pcre2);
    }

    #[test]
    fn pcre2_no_match() {
        test_engine_no_match(EngineKind::Pcre2);
    }

    #[test]
    fn pcre2_case_insensitive() {
        test_engine_case_insensitive(EngineKind::Pcre2);
    }

    #[test]
    fn pcre2_named_captures() {
        test_engine_named_captures(EngineKind::Pcre2);
    }

    #[test]
    fn pcre2_compile_error() {
        test_engine_compile_error(EngineKind::Pcre2);
    }

    #[test]
    fn pcre2_multiline() {
        test_engine_multiline_matching(EngineKind::Pcre2);
    }

    #[test]
    fn pcre2_multiline_anchors() {
        test_engine_multiline_line_anchors(EngineKind::Pcre2);
    }

    #[test]
    fn pcre2_dotall() {
        test_engine_dotall_flag(EngineKind::Pcre2);
    }

    #[test]
    fn pcre2_backreference() {
        let engine = create_engine(EngineKind::Pcre2);
        let flags = EngineFlags::default();
        let compiled = engine.compile(r"(\w+) \1", &flags).unwrap();
        let matches = compiled.find_matches("hello hello world").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "hello hello");
    }
}

fn test_engine_multiline_matching(kind: EngineKind) {
    let engine = create_engine(kind);
    let flags = EngineFlags {
        multi_line: true,
        ..Default::default()
    };
    let compiled = engine.compile(r"^\w+$", &flags).unwrap();
    let matches = compiled.find_matches("hello\nworld\nfoo").unwrap();
    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].text, "hello");
    assert_eq!(matches[1].text, "world");
    assert_eq!(matches[2].text, "foo");
}

fn test_engine_multiline_line_anchors(kind: EngineKind) {
    let engine = create_engine(kind);
    let flags = EngineFlags {
        multi_line: true,
        ..Default::default()
    };
    let compiled = engine.compile(r"^line\d+$", &flags).unwrap();
    let matches = compiled
        .find_matches("line1\nno match\nline42\nline100")
        .unwrap();
    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].text, "line1");
    assert_eq!(matches[1].text, "line42");
    assert_eq!(matches[2].text, "line100");
}

fn test_engine_dotall_flag(kind: EngineKind) {
    let engine = create_engine(kind);
    let flags = EngineFlags {
        dot_matches_newline: true,
        ..Default::default()
    };
    let compiled = engine.compile(r"a.b", &flags).unwrap();
    let matches = compiled.find_matches("a\nb").unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].text, "a\nb");
}

#[test]
fn rust_regex_multiline() {
    test_engine_multiline_matching(EngineKind::RustRegex);
}

#[test]
fn rust_regex_multiline_anchors() {
    test_engine_multiline_line_anchors(EngineKind::RustRegex);
}

#[test]
fn rust_regex_dotall() {
    test_engine_dotall_flag(EngineKind::RustRegex);
}

#[test]
fn fancy_regex_multiline() {
    test_engine_multiline_matching(EngineKind::FancyRegex);
}

#[test]
fn fancy_regex_multiline_anchors() {
    test_engine_multiline_line_anchors(EngineKind::FancyRegex);
}

#[test]
fn fancy_regex_dotall() {
    test_engine_dotall_flag(EngineKind::FancyRegex);
}

#[test]
fn cross_engine_consistency() {
    let pattern = r"\b\w+\b";
    let text = "hello world foo";
    let flags = EngineFlags::default();

    let engines: Vec<EngineKind> = vec![EngineKind::RustRegex, EngineKind::FancyRegex];

    let results: Vec<Vec<String>> = engines
        .iter()
        .map(|kind| {
            let engine = create_engine(*kind);
            let compiled = engine.compile(pattern, &flags).unwrap();
            let matches = compiled.find_matches(text).unwrap();
            matches.into_iter().map(|m| m.text).collect()
        })
        .collect();

    for i in 1..results.len() {
        assert_eq!(
            results[0], results[i],
            "Engine results differ between {:?} and {:?}",
            engines[0], engines[i]
        );
    }
}

// --- Unicode edge cases ---

fn test_engine_unicode_emoji(kind: EngineKind) {
    let engine = create_engine(kind);
    let flags = EngineFlags {
        unicode: true,
        ..Default::default()
    };
    let compiled = engine.compile(r"\p{Emoji}", &flags).unwrap();
    let matches = compiled.find_matches("hello 🎉 world 🚀").unwrap();
    assert!(matches.len() >= 2, "Should match emoji characters");
}

fn test_engine_unicode_cjk(kind: EngineKind) {
    let engine = create_engine(kind);
    let flags = EngineFlags {
        unicode: true,
        ..Default::default()
    };
    let compiled = engine.compile(r"\p{Han}+", &flags).unwrap();
    let matches = compiled.find_matches("hello 你好世界 world").unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].text, "你好世界");
}

fn test_engine_unicode_combining_marks(kind: EngineKind) {
    let engine = create_engine(kind);
    let flags = EngineFlags {
        unicode: true,
        ..Default::default()
    };
    // Match a base letter followed by combining marks
    let compiled = engine.compile(r"e\p{M}", &flags).unwrap();
    // e + combining acute accent (U+0301)
    let text = "e\u{0301}";
    let matches = compiled.find_matches(text).unwrap();
    assert_eq!(matches.len(), 1);
}

fn test_engine_empty_pattern(kind: EngineKind) {
    let engine = create_engine(kind);
    let flags = EngineFlags::default();
    // Empty pattern should compile and not error (returns empty matches)
    let result = engine.compile("", &flags);
    assert!(result.is_ok());
}

fn test_engine_empty_test_string(kind: EngineKind) {
    let engine = create_engine(kind);
    let flags = EngineFlags::default();
    let compiled = engine.compile(r"\d+", &flags).unwrap();
    let matches = compiled.find_matches("").unwrap();
    assert_eq!(matches.len(), 0);
}

#[test]
fn rust_regex_unicode_emoji() {
    test_engine_unicode_emoji(EngineKind::RustRegex);
}

#[test]
fn rust_regex_unicode_cjk() {
    test_engine_unicode_cjk(EngineKind::RustRegex);
}

#[test]
fn rust_regex_unicode_combining_marks() {
    test_engine_unicode_combining_marks(EngineKind::RustRegex);
}

#[test]
fn rust_regex_empty_pattern() {
    test_engine_empty_pattern(EngineKind::RustRegex);
}

#[test]
fn rust_regex_empty_test_string() {
    test_engine_empty_test_string(EngineKind::RustRegex);
}

#[test]
fn fancy_regex_unicode_emoji() {
    test_engine_unicode_emoji(EngineKind::FancyRegex);
}

#[test]
fn fancy_regex_unicode_cjk() {
    test_engine_unicode_cjk(EngineKind::FancyRegex);
}

#[test]
fn fancy_regex_unicode_combining_marks() {
    test_engine_unicode_combining_marks(EngineKind::FancyRegex);
}

#[test]
fn fancy_regex_empty_pattern() {
    test_engine_empty_pattern(EngineKind::FancyRegex);
}

#[test]
fn fancy_regex_empty_test_string() {
    test_engine_empty_test_string(EngineKind::FancyRegex);
}

/// End-to-end test for the `-p --engine fancy` + lookaround path.
///
/// Regression guard for a bug where the `-p` batch path short-circuited on
/// any `self.error` being set, and the `explain::explain(&pattern)` pass
/// (which uses `regex-syntax`, which doesn't speak lookaround) would clobber
/// the error field even after a successful compile. Before the fix, this
/// invocation returned exit code 2 with a misleading "look-around not
/// supported" message even though fancy-regex handled the pattern fine.
mod cli_e2e {
    use std::io::Write as _;
    use std::process::{Command, Stdio};

    fn rgx_bin() -> std::path::PathBuf {
        let mut p = std::env::current_exe().unwrap();
        p.pop();
        if p.ends_with("deps") {
            p.pop();
        }
        p.push(if cfg!(windows) { "rgx.exe" } else { "rgx" });
        p
    }

    fn run_print(args: &[&str], stdin: &str) -> (i32, String, String) {
        let bin = rgx_bin();
        assert!(
            bin.exists(),
            "rgx binary not found at {bin:?}; run `cargo build` first"
        );
        let mut child = Command::new(&bin)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        if !stdin.is_empty() {
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(stdin.as_bytes())
                .unwrap();
        }
        let out = child.wait_with_output().unwrap();
        (
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    #[test]
    fn fancy_lookahead_print_mode_no_explain_clobber() {
        let (code, stdout, _stderr) = run_print(
            &[
                "-p",
                "--engine",
                "fancy",
                "-t",
                "alice@example.com bob@test.org",
                r"\w+(?=@)",
            ],
            "",
        );
        assert_eq!(
            code, 0,
            "fancy + lookahead should exit 0 in -p mode, got stderr={_stderr}"
        );
        assert!(
            stdout.contains("alice"),
            "stdout should contain 'alice', got {stdout:?}"
        );
        assert!(
            stdout.contains("bob"),
            "stdout should contain 'bob', got {stdout:?}"
        );
    }

    #[test]
    fn fancy_lookbehind_print_mode_no_explain_clobber() {
        let (code, stdout, _stderr) = run_print(
            &[
                "-p",
                "--engine",
                "fancy",
                "-t",
                "abc user@host def",
                r"(?<=@)\w+",
            ],
            "",
        );
        assert_eq!(code, 0, "fancy + lookbehind should exit 0 in -p mode");
        assert!(
            stdout.contains("host"),
            "stdout should contain 'host', got {stdout:?}"
        );
    }

    #[test]
    fn fancy_lookahead_print_mode_via_stdin() {
        // Same test path but feed text via stdin instead of -t.
        let (code, stdout, _stderr) = run_print(
            &["-p", "--engine", "fancy", r"\w+(?=@)"],
            "alice@example.com bob@test.org\n",
        );
        assert_eq!(code, 0);
        assert!(stdout.contains("alice"));
        assert!(stdout.contains("bob"));
    }

    #[test]
    fn real_compile_error_still_surfaces_in_print_mode() {
        // Invariant: a genuinely broken pattern must still exit 2 with the
        // engine's compile error. The fix for the explain-clobber bug
        // weakens the error-setting in the explain path, so this test
        // makes sure we didn't accidentally swallow real compile errors.
        let (code, _stdout, stderr) = run_print(
            &["-p", "--engine", "fancy", "-t", "anything", "(unclosed"],
            "",
        );
        assert_eq!(code, 2, "broken pattern should exit 2");
        assert!(
            stderr.to_lowercase().contains("error"),
            "stderr should mention error, got {stderr:?}"
        );
    }
}
