pub mod editor;
pub mod handler;
pub mod vim;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// `#[non_exhaustive]` so future additive variants (e.g. new key actions) are
// non-breaking under semver. Without it, each `Action` variant addition forces
// a minor bump because downstream `match` expressions would no longer be
// exhaustive. Adding this attribute is itself a one-time breaking change.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    InsertChar(char),
    InsertNewline,
    DeleteBack,
    DeleteForward,
    DeleteCharAtCursor,
    DeleteLine,
    ChangeLine,
    OpenLineBelow,
    OpenLineAbove,
    MoveToFirstNonBlank,
    MoveToFirstLine,
    MoveToLastLine,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorHome,
    MoveCursorEnd,
    MoveCursorWordLeft,
    MoveCursorWordRight,
    MoveCursorWordForwardEnd,
    ScrollUp,
    ScrollDown,
    SwitchPanel,
    SwitchPanelBack,
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
    PasteClipboard,
    ToggleWhitespace,
    ToggleQuickref,
    ScrollQuickrefUp,
    ScrollQuickrefDown,
    OutputAndQuit,
    SaveWorkspace,
    OpenRecipes,
    OpenGrex,
    Benchmark,
    ExportRegex101,
    GenerateCode,
    EnterInsertMode,
    EnterInsertModeAppend,
    EnterInsertModeLineStart,
    EnterInsertModeLineEnd,
    EnterNormalMode,
    ToggleDebugger,
    AnalyzeRedos,
    Quit,
    None,
}

pub fn key_to_action(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Esc => Action::Quit,
        KeyCode::Tab => Action::SwitchPanel,
        KeyCode::BackTab => Action::SwitchPanelBack,
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
        KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::OutputAndQuit
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::SaveWorkspace
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::OpenRecipes,
        KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::OpenGrex,
        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Benchmark,
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::ExportRegex101
        }
        KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::GenerateCode,
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::ToggleDebugger
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::AnalyzeRedos,
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
        KeyCode::F(3) => Action::ToggleQuickref,
        KeyCode::PageUp => Action::ScrollQuickrefUp,
        KeyCode::PageDown => Action::ScrollQuickrefDown,
        // Unmapped Ctrl+key / Alt+key chords do nothing. Ctrl+Alt together is
        // AltGr on many layouts (e.g. `@` on German keyboards), so it still types.
        KeyCode::Char(c)
            if key.modifiers.contains(KeyModifiers::CONTROL)
                == key.modifiers.contains(KeyModifiers::ALT) =>
        {
            Action::InsertChar(c)
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn char_key(c: char, modifiers: KeyModifiers) -> Action {
        key_to_action(KeyEvent::new(KeyCode::Char(c), modifiers))
    }

    #[test]
    fn plain_and_shifted_chars_insert() {
        assert_eq!(char_key('a', KeyModifiers::NONE), Action::InsertChar('a'));
        assert_eq!(char_key('A', KeyModifiers::SHIFT), Action::InsertChar('A'));
    }

    #[test]
    fn unmapped_ctrl_or_alt_chords_do_not_insert() {
        assert_eq!(char_key('k', KeyModifiers::CONTROL), Action::None);
        assert_eq!(char_key('f', KeyModifiers::ALT), Action::None);
    }

    #[test]
    fn altgr_chars_still_insert() {
        let altgr = KeyModifiers::CONTROL | KeyModifiers::ALT;
        assert_eq!(char_key('@', altgr), Action::InsertChar('@'));
    }
}
