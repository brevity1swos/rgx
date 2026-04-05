use rgx::config::settings::Settings;
use rgx::config::workspace::Workspace;
use rgx::engine::{EngineFlags, EngineKind};

#[test]
fn test_settings_defaults() {
    let settings = Settings::default();
    assert_eq!(settings.default_engine, "rust");
    assert!(!settings.case_insensitive);
    assert!(!settings.multiline);
    assert!(!settings.dotall);
    assert!(settings.unicode); // default true
    assert!(!settings.extended);
    assert!(!settings.show_whitespace);
}

#[test]
fn test_settings_parse_engine() {
    let settings = Settings {
        default_engine: "rust".to_string(),
        ..Default::default()
    };
    assert_eq!(settings.parse_engine(), EngineKind::RustRegex);

    let settings = Settings {
        default_engine: "fancy".to_string(),
        ..Default::default()
    };
    assert_eq!(settings.parse_engine(), EngineKind::FancyRegex);

    let settings = Settings {
        default_engine: "unknown".to_string(),
        ..Default::default()
    };
    assert_eq!(settings.parse_engine(), EngineKind::RustRegex);
}

#[test]
fn test_settings_partial_toml() {
    let toml_str = r#"
default_engine = "fancy"
case_insensitive = true
"#;
    let settings: Settings = toml::from_str(toml_str).unwrap();
    assert_eq!(settings.default_engine, "fancy");
    assert!(settings.case_insensitive);
    // Other fields should have defaults
    assert!(!settings.multiline);
    assert!(settings.unicode); // default true
    assert!(!settings.show_whitespace);
}

#[test]
fn test_settings_full_toml() {
    let toml_str = r#"
default_engine = "rust"
case_insensitive = false
multiline = true
dotall = true
unicode = false
extended = true
show_whitespace = true
"#;
    let settings: Settings = toml::from_str(toml_str).unwrap();
    assert_eq!(settings.default_engine, "rust");
    assert!(!settings.case_insensitive);
    assert!(settings.multiline);
    assert!(settings.dotall);
    assert!(!settings.unicode);
    assert!(settings.extended);
    assert!(settings.show_whitespace);
}

#[test]
fn test_settings_empty_toml() {
    let settings: Settings = toml::from_str("").unwrap();
    assert_eq!(settings.default_engine, "rust");
    assert!(settings.unicode);
}

#[test]
fn test_workspace_round_trip() {
    let flags = EngineFlags {
        case_insensitive: true,
        multi_line: false,
        dot_matches_newline: true,
        unicode: true,
        extended: false,
    };
    let mut app = rgx::app::App::new(EngineKind::FancyRegex, flags);
    app.set_pattern(r"\d{3}-\d{4}");
    app.set_test_string("Call 555-1234 today");
    app.set_replacement("XXX-XXXX");

    let ws = Workspace::from_app(&app);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.toml");
    ws.save(&path).unwrap();

    let loaded = Workspace::load(&path).unwrap();
    assert_eq!(loaded.pattern, r"\d{3}-\d{4}");
    assert_eq!(loaded.test_string, "Call 555-1234 today");
    assert_eq!(loaded.replacement, "XXX-XXXX");
    assert_eq!(loaded.engine, "fancy");
    assert!(loaded.case_insensitive);
    assert!(!loaded.multiline);
    assert!(loaded.dotall);
    assert!(loaded.unicode);
    assert!(!loaded.extended);
}

#[test]
fn test_workspace_apply_restores_state() {
    let flags = EngineFlags::default();
    let mut app = rgx::app::App::new(EngineKind::RustRegex, flags);

    let toml_str = r#"
pattern = "hello+"
test_string = "hellooo world"
replacement = ""
engine = "fancy"
case_insensitive = true
multiline = false
dotall = false
unicode = true
extended = false
show_whitespace = true
"#;
    let ws: Workspace = toml::from_str(toml_str).unwrap();
    ws.apply(&mut app);

    assert_eq!(app.regex_editor.content(), "hello+");
    assert_eq!(app.test_editor.content(), "hellooo world");
    assert_eq!(app.engine_kind, EngineKind::FancyRegex);
    assert!(app.flags.case_insensitive);
    assert!(app.show_whitespace);
}
