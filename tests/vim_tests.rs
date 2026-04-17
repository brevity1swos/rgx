use rgx::app::App;
use rgx::engine::{EngineFlags, EngineKind};
use rgx::input::vim::{vim_key_to_action, VimMode, VimState};

fn create_app() -> App {
    let mut app = App::new(EngineKind::RustRegex, EngineFlags::default());
    app.vim_mode = true;
    app
}

#[test]
fn vim_mode_disabled_by_default() {
    let app = App::new(EngineKind::RustRegex, EngineFlags::default());
    assert!(!app.vim_mode);
}

#[test]
fn vim_mode_initial_state() {
    let app = create_app();
    assert_eq!(app.vim_state.mode, VimMode::Normal);
}

#[test]
fn vim_state_mode_transitions() {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    let mut state = VimState::new();
    assert_eq!(state.mode, VimMode::Normal);

    let key = |code| KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    // i -> Insert
    vim_key_to_action(key(KeyCode::Char('i')), &mut state);
    assert_eq!(state.mode, VimMode::Insert);

    // Esc -> Normal
    vim_key_to_action(key(KeyCode::Esc), &mut state);
    assert_eq!(state.mode, VimMode::Normal);
}

#[test]
fn vim_render_with_vim_mode() {
    use ratatui::{backend::TestBackend, Terminal};
    let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
    let mut app = create_app();
    app.set_test_string("hello world");
    app.set_pattern(r"\w+");
    terminal.draw(|frame| rgx::ui::render(frame, &app)).unwrap();
}

#[test]
fn vim_editor_delete_line() {
    let mut app = create_app();
    app.set_test_string("line1\nline2\nline3");
    app.test_editor.set_cursor_by_position(1, 0);
    app.test_editor.delete_line();
    assert_eq!(app.test_editor.content(), "line1\nline3");
}

#[test]
fn vim_editor_clear_line() {
    let mut app = create_app();
    app.set_test_string("line1\nline2\nline3");
    app.test_editor.set_cursor_by_position(1, 0);
    app.test_editor.clear_line();
    assert_eq!(app.test_editor.content(), "line1\n\nline3");
}

#[test]
fn vim_editor_insert_str() {
    let mut app = create_app();
    app.set_test_string("hd");
    app.test_editor.set_cursor_by_col(1);
    app.test_editor.insert_str("ello worl");
    assert_eq!(app.test_editor.content(), "hello world");
}

#[test]
fn ctrl_x_opens_grex_overlay_in_vim_normal_mode() {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use rgx::input::Action;

    let mut state = VimState::new();
    assert_eq!(state.mode, VimMode::Normal);

    let key = KeyEvent {
        code: KeyCode::Char('x'),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    let action = vim_key_to_action(key, &mut state);
    assert_eq!(action, Action::OpenGrex);
    // Mode should remain Normal (Ctrl+X is a global shortcut, not a mode-changer).
    assert_eq!(state.mode, VimMode::Normal);
}

#[test]
fn plain_x_still_deletes_char_in_vim_normal_mode() {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use rgx::input::Action;

    let mut state = VimState::new();
    let key = KeyEvent {
        code: KeyCode::Char('x'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    let action = vim_key_to_action(key, &mut state);
    assert_eq!(action, Action::DeleteCharAtCursor);
}
