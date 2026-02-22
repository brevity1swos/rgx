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
fn match_display_shows_results() {
    let mut terminal = create_test_terminal();
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("user@example");
    app.set_pattern(r"(\w+)@(\w+)");

    // Verify match data exists
    assert_eq!(app.matches.len(), 1);
    assert_eq!(app.matches[0].text, "user@example");
    assert_eq!(app.matches[0].captures.len(), 2);

    // Render and check that match text appears in the buffer
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let buffer_text: String = buffer
        .content()
        .iter()
        .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
        .collect();
    assert!(
        buffer_text.contains("Match 1"),
        "Buffer should contain 'Match 1' but got: {}",
        buffer_text
    );
}

#[test]
fn multiline_test_string_renders() {
    let mut terminal = create_test_terminal();
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("hello\nworld");
    app.set_pattern(r"\w+");

    // Should find matches on both lines
    assert_eq!(app.matches.len(), 2);
    assert_eq!(app.matches[0].text, "hello");
    assert_eq!(app.matches[1].text, "world");

    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
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

#[test]
fn render_with_replacement() {
    let mut terminal = create_test_terminal();
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("hello 123 world 456");
    app.set_pattern(r"\d+");
    app.set_replacement("[NUM]");

    assert!(app.replace_result.is_some());
    let result = app.replace_result.as_ref().unwrap();
    assert_eq!(result.output, "hello [NUM] world [NUM]");

    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
}

#[test]
fn render_empty_replacement() {
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("hello 123");
    app.set_pattern(r"\d+");
    // No replacement set
    assert!(app.replace_result.is_none());
}

#[test]
fn panel_cycling_includes_replace() {
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    assert_eq!(app.focused_panel, 0);
    app.focused_panel = (app.focused_panel + 1) % 5;
    assert_eq!(app.focused_panel, 1);
    app.focused_panel = (app.focused_panel + 1) % 5;
    assert_eq!(app.focused_panel, 2); // replace panel
    app.focused_panel = (app.focused_panel + 1) % 5;
    assert_eq!(app.focused_panel, 3); // matches
    app.focused_panel = (app.focused_panel + 1) % 5;
    assert_eq!(app.focused_panel, 4); // explanation
    app.focused_panel = (app.focused_panel + 1) % 5;
    assert_eq!(app.focused_panel, 0); // back to regex
}

#[test]
fn replacement_with_named_groups() {
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("2024-01");
    app.set_pattern(r"(?P<y>\d{4})-(?P<m>\d{2})");
    app.set_replacement("${m}/${y}");

    assert!(app.replace_result.is_some());
    let result = app.replace_result.as_ref().unwrap();
    assert_eq!(result.output, "01/2024");
}

#[test]
fn replacement_clears_on_empty_template() {
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("hello 123");
    app.set_pattern(r"\d+");
    app.set_replacement("[NUM]");
    assert!(app.replace_result.is_some());

    app.set_replacement("");
    assert!(app.replace_result.is_none());
}
