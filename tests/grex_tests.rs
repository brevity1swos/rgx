use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rgx::app::{App, OverlayState};
use rgx::engine::{EngineFlags, EngineKind};
use rgx::grex_integration::{generate, GrexOptions};
use rgx::input::{key_to_action, Action};

fn new_test_app() -> App {
    App::new(EngineKind::RustRegex, EngineFlags::default())
}

#[test]
fn ctrl_x_maps_to_open_grex() {
    let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL);
    assert_eq!(key_to_action(key), Action::OpenGrex);
}

#[test]
fn open_grex_action_opens_overlay() {
    let mut app = new_test_app();
    assert!(app.overlay.grex.is_none());
    app.handle_action(Action::OpenGrex, 10_000);
    assert!(app.overlay.grex.is_some());
}

#[test]
fn overlay_state_default_has_no_grex_overlay() {
    let overlay = OverlayState::default();
    assert!(overlay.grex.is_none());
}

#[test]
fn default_options_match_spec_defaults() {
    let opts = GrexOptions::default();
    assert!(opts.digit);
    assert!(opts.anchors);
    assert!(!opts.case_insensitive);
}

#[test]
fn empty_input_returns_empty_string() {
    let result = generate(&[], GrexOptions::default());
    assert_eq!(result, "");
}

#[test]
fn single_example_with_defaults_is_anchored_literal() {
    let examples = vec!["hello".to_string()];
    let result = generate(&examples, GrexOptions::default());
    assert!(result.starts_with('^'), "expected leading ^ in {result}");
    assert!(result.ends_with('$'), "expected trailing $ in {result}");
    assert!(result.contains("hello"), "expected literal in {result}");
}

#[test]
fn digit_flag_generates_digit_class() {
    let examples = vec!["a1".to_string(), "b22".to_string(), "c333".to_string()];
    let result = generate(
        &examples,
        GrexOptions {
            digit: true,
            anchors: true,
            case_insensitive: false,
        },
    );
    assert!(result.contains(r"\d"), "expected \\d in {result}");
}

#[test]
fn anchors_off_produces_unanchored_pattern() {
    let examples = vec!["hello".to_string()];
    let result = generate(
        &examples,
        GrexOptions {
            digit: false,
            anchors: false,
            case_insensitive: false,
        },
    );
    assert!(
        !result.starts_with('^'),
        "expected no leading ^ in {result}"
    );
    assert!(!result.ends_with('$'), "expected no trailing $ in {result}");
}

#[test]
fn case_insensitive_flag_adds_case_modifier() {
    let examples = vec![
        "Hello".to_string(),
        "HELLO".to_string(),
        "hello".to_string(),
    ];
    let result = generate(
        &examples,
        GrexOptions {
            digit: false,
            anchors: true,
            case_insensitive: true,
        },
    );
    assert!(result.contains("(?i)"), "expected (?i) in {result}");
}
