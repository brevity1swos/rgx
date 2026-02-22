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
    fn pcre2_backreference() {
        let engine = create_engine(EngineKind::Pcre2);
        let flags = EngineFlags::default();
        let compiled = engine.compile(r"(\w+) \1", &flags).unwrap();
        let matches = compiled.find_matches("hello hello world").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "hello hello");
    }
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
