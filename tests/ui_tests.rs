use ratatui::{backend::TestBackend, Terminal};
use rgx::app::App;
use rgx::engine::{EngineFlags, EngineKind};
use rgx::ui;

fn create_test_terminal() -> Terminal<TestBackend> {
    let backend = TestBackend::new(80, 24);
    Terminal::new(backend).unwrap()
}

#[test]
fn render_empty_state() {
    let mut terminal = create_test_terminal();
    let app = App::new(EngineKind::RustRegex, EngineFlags::default());
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
}

#[test]
fn render_with_pattern() {
    let mut terminal = create_test_terminal();
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("hello 123 world 456");
    app.set_pattern(r"\d+");
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
    assert_eq!(app.matches.len(), 2);
}

#[test]
fn render_with_error() {
    let mut terminal = create_test_terminal();
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_pattern(r"(unclosed");
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
    assert!(app.error.is_some());
}

#[test]
fn render_with_captures() {
    let mut terminal = create_test_terminal();
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("user@example.com");
    app.set_pattern(r"(\w+)@(\w+)\.(\w+)");
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
    assert_eq!(app.matches.len(), 1);
    assert_eq!(app.matches[0].captures.len(), 3);
}

#[test]
fn render_help_overlay() {
    let mut terminal = create_test_terminal();
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.show_help = true;
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
}

#[test]
fn engine_switching() {
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("hello 123");
    app.set_pattern(r"\d+");
    assert_eq!(app.matches.len(), 1);

    app.switch_engine();
    assert_eq!(app.engine_kind, EngineKind::FancyRegex);
    assert_eq!(app.matches.len(), 1);
}

#[test]
fn flag_toggles() {
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("Hello HELLO hello");
    app.set_pattern("hello");
    assert_eq!(app.matches.len(), 1); // only lowercase match

    app.flags.toggle_case_insensitive();
    app.recompute();
    assert_eq!(app.matches.len(), 3); // all match now
}

#[test]
fn narrow_terminal_layout() {
    // Test that narrow terminals don't crash
    let backend = TestBackend::new(40, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("test");
    app.set_pattern(r"\w+");
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
}
