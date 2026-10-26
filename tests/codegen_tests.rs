use rgx::codegen::{generate_code, Language};
use rgx::engine::EngineFlags;

fn plain() -> EngineFlags {
    EngineFlags {
        unicode: false,
        ..EngineFlags::default()
    }
}

#[test]
fn rust_raw_string_keeps_backslashes() {
    let code = generate_code(&Language::Rust, r"\d+", &plain());
    assert!(code.contains(r#"Regex::new(r"\d+")"#), "{code}");
}

#[test]
fn rust_raw_string_grows_hashes_for_quotes() {
    let code = generate_code(&Language::Rust, r##"say "hi"#"##, &plain());
    assert!(code.contains(r###"r##"say "hi"#"##"###), "{code}");
}

#[test]
fn python_raw_string_keeps_backslashes() {
    let code = generate_code(&Language::Python, r"\d+", &plain());
    assert!(code.contains(r#"re.compile(r"\d+")"#), "{code}");
}

#[test]
fn python_switches_quote_style_for_double_quotes() {
    let code = generate_code(&Language::Python, r#"\w+="x""#, &plain());
    assert!(code.contains(r#"re.compile(r'\w+="x"')"#), "{code}");
}

#[test]
fn python_falls_back_to_escaped_string_with_both_quotes() {
    let code = generate_code(&Language::Python, r#"'"\d"#, &plain());
    assert!(code.contains(r#"re.compile("'\"\\d")"#), "{code}");
}

#[test]
fn go_unicode_flag_does_not_emit_ungreedy() {
    let code = generate_code(&Language::Go, "a+", &EngineFlags::default());
    assert!(code.contains("regexp.MustCompile(`a+`)"), "{code}");
}

#[test]
fn javascript_does_not_double_escape_slash() {
    let code = generate_code(&Language::JavaScript, r"a\/b/c", &plain());
    assert!(code.contains(r"/a\/b\/c/g"), "{code}");
}

#[test]
fn javascript_escapes_literal_newline() {
    let code = generate_code(&Language::JavaScript, "a\nb", &plain());
    assert!(code.contains(r"/a\nb/g"), "{code}");
}

#[test]
fn php_preserves_backslashes_and_quotes() {
    let code = generate_code(&Language::Php, r"\\it's/", &plain());
    assert!(code.contains(r"'/\\\\it\'s\\//'"), "{code}");
}

#[test]
fn ruby_multiline_flag_is_not_dotall() {
    let flags = EngineFlags {
        multi_line: true,
        ..plain()
    };
    let code = generate_code(&Language::Ruby, "^a", &flags);
    assert!(code.contains("pattern = /^a/\n"), "{code}");

    let flags = EngineFlags {
        dot_matches_newline: true,
        ..plain()
    };
    let code = generate_code(&Language::Ruby, "a.b", &flags);
    assert!(code.contains("pattern = /a.b/m\n"), "{code}");
}

#[test]
fn java_escapes_literal_newline() {
    let code = generate_code(&Language::Java, "a\nb", &plain());
    assert!(code.contains(r#"Pattern.compile("a\nb")"#), "{code}");
}

#[test]
fn ruby_escapes_interpolation_sequences() {
    let code = generate_code(
        &Language::Ruby,
        r#"a#{system("id")}#@x#$y#z\#{k}"#,
        &plain(),
    );
    assert!(
        code.contains(r#"pattern = /a\#{system("id")}\#@x\#$y#z\#{k}/"#),
        "{code}"
    );
}
