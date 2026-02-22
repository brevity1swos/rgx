use rgx::config::settings::Settings;
use rgx::engine::EngineKind;

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
    assert!(!settings.theme.catppuccin);
}

#[test]
fn test_settings_parse_engine() {
    let mut settings = Settings::default();

    settings.default_engine = "rust".to_string();
    assert_eq!(settings.parse_engine(), EngineKind::RustRegex);

    settings.default_engine = "fancy".to_string();
    assert_eq!(settings.parse_engine(), EngineKind::FancyRegex);

    settings.default_engine = "unknown".to_string();
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

[theme]
catppuccin = true
"#;
    let settings: Settings = toml::from_str(toml_str).unwrap();
    assert_eq!(settings.default_engine, "rust");
    assert!(!settings.case_insensitive);
    assert!(settings.multiline);
    assert!(settings.dotall);
    assert!(!settings.unicode);
    assert!(settings.extended);
    assert!(settings.show_whitespace);
    assert!(settings.theme.catppuccin);
}

#[test]
fn test_settings_empty_toml() {
    let settings: Settings = toml::from_str("").unwrap();
    assert_eq!(settings.default_engine, "rust");
    assert!(settings.unicode);
}
