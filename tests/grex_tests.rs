use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use rgx::app::{App, OverlayState};
use rgx::engine::{EngineFlags, EngineKind};
use rgx::grex_integration::{generate, GrexOptions};
use rgx::input::{key_to_action, Action};
use rgx::ui;
use rgx::ui::grex_overlay::GrexOverlayState;

fn new_test_app() -> App {
    App::new(EngineKind::RustRegex, EngineFlags::default())
}

fn new_test_terminal() -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(80, 24)).unwrap()
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
fn grex_overlay_renders_empty_state_without_panic() {
    let mut terminal = new_test_terminal();
    let state = GrexOverlayState::default();
    terminal
        .draw(|frame| {
            let area = frame.area();
            ui::grex_overlay::render(frame, area, &state);
        })
        .unwrap();
    // The overlay should draw the placeholder. We verify by scanning the buffer.
    let buffer = terminal.backend().buffer().clone();
    let rendered: String = buffer
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<Vec<_>>()
        .join("");
    assert!(
        rendered.contains("Enter one example per line"),
        "empty placeholder missing: {rendered}"
    );
    assert!(
        rendered.contains("(none yet)"),
        "pattern placeholder missing"
    );
    assert!(rendered.contains("[D]igit"), "digit flag label missing");
    assert!(rendered.contains("[A]nchors"), "anchors flag label missing");
}

#[test]
fn grex_overlay_renders_populated_state() {
    let mut terminal = new_test_terminal();
    let mut state = GrexOverlayState::default();
    state.editor.insert_str("foo\nbar\nbaz");
    state.generated_pattern = Some("^(?:foo|bar|baz)$".to_string());
    terminal
        .draw(|frame| {
            let area = frame.area();
            ui::grex_overlay::render(frame, area, &state);
        })
        .unwrap();
    let buffer = terminal.backend().buffer().clone();
    let rendered: String = buffer
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<Vec<_>>()
        .join("");
    assert!(rendered.contains("foo"), "example line missing");
    assert!(
        rendered.contains("^(?:foo|bar|baz)$"),
        "pattern preview missing"
    );
    assert!(!rendered.contains("(none yet)"), "empty placeholder leaked");
}

fn press(key: KeyCode, mods: KeyModifiers) -> KeyEvent {
    KeyEvent::new(key, mods)
}

#[test]
fn tab_with_generated_pattern_loads_into_regex_editor_and_closes_overlay() {
    let mut app = new_test_app();
    app.handle_action(Action::OpenGrex, 10_000);
    if let Some(overlay) = app.overlay.grex.as_mut() {
        overlay.generated_pattern = Some("^hello$".to_string());
    }
    app.dispatch_grex_overlay_key(press(KeyCode::Tab, KeyModifiers::NONE));
    assert!(app.overlay.grex.is_none());
    assert_eq!(app.regex_editor.content(), "^hello$");
}

#[test]
fn tab_without_pattern_is_noop() {
    let mut app = new_test_app();
    app.handle_action(Action::OpenGrex, 10_000);
    app.dispatch_grex_overlay_key(press(KeyCode::Tab, KeyModifiers::NONE));
    // Overlay stays open; no pattern loaded.
    assert!(app.overlay.grex.is_some());
}

#[test]
fn esc_closes_grex_overlay_without_loading() {
    let mut app = new_test_app();
    app.handle_action(Action::OpenGrex, 10_000);
    if let Some(overlay) = app.overlay.grex.as_mut() {
        overlay.generated_pattern = Some("should not be loaded".to_string());
    }
    let prior_pattern = app.regex_editor.content().to_string();
    app.dispatch_grex_overlay_key(press(KeyCode::Esc, KeyModifiers::NONE));
    assert!(app.overlay.grex.is_none());
    assert_eq!(app.regex_editor.content(), prior_pattern);
}

#[test]
fn alt_d_toggles_digit_flag() {
    let mut app = new_test_app();
    app.handle_action(Action::OpenGrex, 10_000);
    let before = app.overlay.grex.as_ref().unwrap().options.digit;
    app.dispatch_grex_overlay_key(press(KeyCode::Char('d'), KeyModifiers::ALT));
    let after = app.overlay.grex.as_ref().unwrap().options.digit;
    assert_ne!(before, after);
}

#[test]
fn alt_a_toggles_anchors_flag() {
    let mut app = new_test_app();
    app.handle_action(Action::OpenGrex, 10_000);
    let before = app.overlay.grex.as_ref().unwrap().options.anchors;
    app.dispatch_grex_overlay_key(press(KeyCode::Char('a'), KeyModifiers::ALT));
    let after = app.overlay.grex.as_ref().unwrap().options.anchors;
    assert_ne!(before, after);
}

#[test]
fn alt_c_toggles_case_insensitive_flag() {
    let mut app = new_test_app();
    app.handle_action(Action::OpenGrex, 10_000);
    let before = app.overlay.grex.as_ref().unwrap().options.case_insensitive;
    app.dispatch_grex_overlay_key(press(KeyCode::Char('c'), KeyModifiers::ALT));
    let after = app.overlay.grex.as_ref().unwrap().options.case_insensitive;
    assert_ne!(before, after);
}

#[test]
fn plain_characters_append_to_overlay_editor() {
    let mut app = new_test_app();
    app.handle_action(Action::OpenGrex, 10_000);
    for ch in ['h', 'i'] {
        app.dispatch_grex_overlay_key(press(KeyCode::Char(ch), KeyModifiers::NONE));
    }
    let content = app
        .overlay
        .grex
        .as_ref()
        .unwrap()
        .editor
        .content()
        .to_string();
    assert_eq!(content, "hi");
}

#[test]
fn enter_inserts_newline_in_overlay_editor() {
    let mut app = new_test_app();
    app.handle_action(Action::OpenGrex, 10_000);
    for ch in ['a', 'b'] {
        app.dispatch_grex_overlay_key(press(KeyCode::Char(ch), KeyModifiers::NONE));
    }
    app.dispatch_grex_overlay_key(press(KeyCode::Enter, KeyModifiers::NONE));
    app.dispatch_grex_overlay_key(press(KeyCode::Char('c'), KeyModifiers::NONE));
    let content = app
        .overlay
        .grex
        .as_ref()
        .unwrap()
        .editor
        .content()
        .to_string();
    assert_eq!(content, "ab\nc");
}

#[test]
fn editing_sets_debounce_deadline() {
    let mut app = new_test_app();
    app.handle_action(Action::OpenGrex, 10_000);
    assert!(app
        .overlay
        .grex
        .as_ref()
        .unwrap()
        .debounce_deadline
        .is_none());
    app.dispatch_grex_overlay_key(press(KeyCode::Char('x'), KeyModifiers::NONE));
    assert!(app
        .overlay
        .grex
        .as_ref()
        .unwrap()
        .debounce_deadline
        .is_some());
}

#[test]
fn ui_render_routes_to_grex_overlay_when_open() {
    let mut terminal = new_test_terminal();
    let mut app = new_test_app();
    app.handle_action(Action::OpenGrex, 10_000);
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let rendered: String = buffer
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<Vec<_>>()
        .join("");
    assert!(
        rendered.contains("Generate Regex from Examples"),
        "grex overlay not rendered by ui::render"
    );
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
