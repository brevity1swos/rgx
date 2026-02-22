pub mod editor;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    InsertChar(char),
    InsertNewline,
    DeleteBack,
    DeleteForward,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorHome,
    MoveCursorEnd,
    MoveCursorWordLeft,
    MoveCursorWordRight,
    ScrollUp,
    ScrollDown,
    SwitchPanel,
    SwitchEngine,
    ToggleCaseInsensitive,
    ToggleMultiLine,
    ToggleDotAll,
    ToggleUnicode,
    ToggleExtended,
    ShowHelp,
    Undo,
    Redo,
    HistoryPrev,
    HistoryNext,
    CopyMatch,
    ToggleWhitespace,
    Quit,
    None,
}

pub fn key_to_action(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Esc => Action::Quit,
        KeyCode::Tab => Action::SwitchPanel,
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::SwitchEngine,
        KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Undo,
        KeyCode::Char('Z')
            if key
                .modifiers
                .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) =>
        {
            Action::Redo
        }
        KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::CopyMatch,
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::ToggleWhitespace
        }
        KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::ALT) => {
            Action::ToggleCaseInsensitive
        }
        KeyCode::Char('m') if key.modifiers.contains(KeyModifiers::ALT) => Action::ToggleMultiLine,
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::ALT) => Action::ToggleDotAll,
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::ALT) => Action::ToggleUnicode,
        KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::ALT) => Action::ToggleExtended,
        KeyCode::Up if key.modifiers.contains(KeyModifiers::ALT) => Action::HistoryPrev,
        KeyCode::Down if key.modifiers.contains(KeyModifiers::ALT) => Action::HistoryNext,
        KeyCode::F(1) => Action::ShowHelp,
        KeyCode::Char(c) => Action::InsertChar(c),
        KeyCode::Enter => Action::InsertNewline,
        KeyCode::Backspace => Action::DeleteBack,
        KeyCode::Delete => Action::DeleteForward,
        KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::MoveCursorWordLeft
        }
        KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::MoveCursorWordRight
        }
        KeyCode::Left => Action::MoveCursorLeft,
        KeyCode::Right => Action::MoveCursorRight,
        KeyCode::Up => Action::ScrollUp,
        KeyCode::Down => Action::ScrollDown,
        KeyCode::Home => Action::MoveCursorHome,
        KeyCode::End => Action::MoveCursorEnd,
        _ => Action::None,
    }
}
