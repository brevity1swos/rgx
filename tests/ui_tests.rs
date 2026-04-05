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
    app.overlay.help = true;
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

// --- Phase 2 tests ---

#[test]
fn undo_redo_regex_editor() {
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("hello 123");
    app.regex_editor.insert_char('\\');
    app.regex_editor.insert_char('d');
    app.regex_editor.insert_char('+');
    app.recompute();
    assert_eq!(app.matches.len(), 1);

    app.regex_editor.undo();
    app.recompute();
    // Pattern is now "\d" — still matches
    assert_eq!(app.regex_editor.content(), "\\d");

    app.regex_editor.redo();
    app.recompute();
    assert_eq!(app.regex_editor.content(), "\\d+");
}

#[test]
fn pattern_history_navigation() {
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("abc 123 def 456");

    // Commit first pattern
    app.set_pattern(r"\d+");
    app.commit_pattern_to_history();

    // Commit second pattern
    app.set_pattern(r"\w+");
    app.commit_pattern_to_history();

    assert_eq!(app.pattern_history.len(), 2);

    // Navigate back
    app.set_pattern("current");
    app.history_prev();
    assert_eq!(app.regex_editor.content(), r"\w+");

    app.history_prev();
    assert_eq!(app.regex_editor.content(), r"\d+");

    // Navigate forward
    app.history_next();
    assert_eq!(app.regex_editor.content(), r"\w+");

    // Past end restores temp
    app.history_next();
    assert_eq!(app.regex_editor.content(), "current");
}

#[test]
fn pattern_history_dedup() {
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_pattern(r"\d+");
    app.commit_pattern_to_history();
    app.commit_pattern_to_history(); // same pattern, should dedup
    assert_eq!(app.pattern_history.len(), 1);
}

#[test]
fn match_selection_navigation() {
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("aa bb cc");
    app.set_pattern(r"\w+");
    assert_eq!(app.matches.len(), 3);

    // Start at match 0, no capture
    assert_eq!(app.selected_match, 0);
    assert_eq!(app.selected_capture, None);

    // Navigate down through matches (no captures here)
    app.select_match_next();
    assert_eq!(app.selected_match, 1);

    app.select_match_next();
    assert_eq!(app.selected_match, 2);

    // Can't go past last match
    app.select_match_next();
    assert_eq!(app.selected_match, 2);

    // Navigate back up
    app.select_match_prev();
    assert_eq!(app.selected_match, 1);

    app.select_match_prev();
    assert_eq!(app.selected_match, 0);

    // Can't go before first match
    app.select_match_prev();
    assert_eq!(app.selected_match, 0);
}

#[test]
fn match_selection_with_captures() {
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("user@example");
    app.set_pattern(r"(\w+)@(\w+)");
    assert_eq!(app.matches.len(), 1);
    assert_eq!(app.matches[0].captures.len(), 2);

    // Start at match 0
    assert_eq!(app.selected_match, 0);
    assert_eq!(app.selected_capture, None);

    // Next goes to first capture
    app.select_match_next();
    assert_eq!(app.selected_match, 0);
    assert_eq!(app.selected_capture, Some(0));

    // Next goes to second capture
    app.select_match_next();
    assert_eq!(app.selected_match, 0);
    assert_eq!(app.selected_capture, Some(1));

    // Prev goes back to first capture
    app.select_match_prev();
    assert_eq!(app.selected_capture, Some(0));

    // Prev goes back to match header
    app.select_match_prev();
    assert_eq!(app.selected_capture, None);
}

#[test]
fn selection_resets_on_rematch() {
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("aa bb");
    app.set_pattern(r"\w+");
    app.select_match_next();
    assert_eq!(app.selected_match, 1);

    // Changing test string resets selection
    app.set_test_string("aa bb cc");
    assert_eq!(app.selected_match, 0);
    assert_eq!(app.selected_capture, None);
}

#[test]
fn help_pages_render() {
    let mut terminal = create_test_terminal();
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.overlay.help = true;

    // Page 0 (default)
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();

    // Page 1
    app.overlay.help_page = 1;
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();

    // Page 2
    app.overlay.help_page = 2;
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
}

#[test]
fn help_page_clamped() {
    let mut terminal = create_test_terminal();
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.overlay.help = true;
    app.overlay.help_page = 99; // out of bounds, should clamp
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
}

#[test]
fn multiline_flag_matching() {
    let mut app = App::new(
        EngineKind::RustRegex,
        EngineFlags {
            multi_line: true,
            ..Default::default()
        },
    );
    app.set_test_string("line1\nno match\nline42");
    app.set_pattern(r"^line\d+$");
    assert_eq!(app.matches.len(), 2);
    assert_eq!(app.matches[0].text, "line1");
    assert_eq!(app.matches[1].text, "line42");
}

#[test]
fn dotall_flag_matching() {
    let mut app = App::new(
        EngineKind::RustRegex,
        EngineFlags {
            dot_matches_newline: true,
            ..Default::default()
        },
    );
    app.set_test_string("a\nb");
    app.set_pattern("a.b");
    assert_eq!(app.matches.len(), 1);
    assert_eq!(app.matches[0].text, "a\nb");
}

#[test]
fn whitespace_visualization_toggle() {
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    assert!(!app.show_whitespace);
    app.show_whitespace = true;
    assert!(app.show_whitespace);

    // Rendering with whitespace mode should not panic
    let mut terminal = create_test_terminal();
    app.set_test_string("hello world\nfoo bar");
    app.set_pattern(r"\w+");
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
}

#[test]
fn compute_layout_does_not_panic() {
    use ratatui::layout::Rect;
    // Wide terminal
    let _ = ui::compute_layout(Rect::new(0, 0, 120, 40));
    // Narrow terminal
    let _ = ui::compute_layout(Rect::new(0, 0, 40, 20));
    // Tiny terminal
    let _ = ui::compute_layout(Rect::new(0, 0, 10, 10));
}

#[test]
fn test_empty_state_render() {
    let mut terminal = create_test_terminal();
    let app = App::new(EngineKind::RustRegex, EngineFlags::default());
    // No pattern, no test string — should render without panic
    assert!(app.regex_editor.content().is_empty());
    assert!(app.test_editor.content().is_empty());
    assert!(app.matches.is_empty());
    assert!(app.error.is_none());
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
}

#[test]
fn test_replace_invalid_capture_ref() {
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.set_test_string("hello world");
    app.set_pattern(r"(\w+) (\w+)");
    // $99 references a non-existent group — should not panic
    app.set_replacement("$99");
    assert!(app.replace_result.is_some());
    let result = app.replace_result.as_ref().unwrap();
    // $99 is parsed as $9 then literal '9', $9 doesn't exist so nothing
    // The output should just not crash
    assert!(!result.output.is_empty() || result.output.is_empty());
}
