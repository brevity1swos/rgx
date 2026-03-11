use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{key_to_action, Action};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    Normal,
    Insert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingKey {
    None,
    G,
    D,
    C,
}

#[derive(Debug, Clone)]
pub struct VimState {
    pub mode: VimMode,
    pending: PendingKey,
}

impl VimState {
    pub fn new() -> Self {
        Self {
            mode: VimMode::Normal,
            pending: PendingKey::None,
        }
    }
}

impl Default for VimState {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns true if this key should bypass vim processing.
fn is_global_shortcut(key: &KeyEvent) -> bool {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

    if ctrl {
        return matches!(
            key.code,
            KeyCode::Char('e')
                | KeyCode::Char('z')
                | KeyCode::Char('Z')
                | KeyCode::Char('y')
                | KeyCode::Char('w')
                | KeyCode::Char('o')
                | KeyCode::Char('s')
                | KeyCode::Char('r')
                | KeyCode::Char('b')
                | KeyCode::Char('c')
                | KeyCode::Char('q')
                | KeyCode::Left
                | KeyCode::Right
        );
    }
    if alt {
        return matches!(
            key.code,
            KeyCode::Char('i')
                | KeyCode::Char('m')
                | KeyCode::Char('s')
                | KeyCode::Char('u')
                | KeyCode::Char('x')
                | KeyCode::Up
                | KeyCode::Down
        );
    }
    matches!(key.code, KeyCode::F(1) | KeyCode::Tab | KeyCode::BackTab)
}

/// Process a key event through the vim state machine.
pub fn vim_key_to_action(key: KeyEvent, state: &mut VimState) -> Action {
    if is_global_shortcut(&key) {
        state.pending = PendingKey::None;
        return key_to_action(key);
    }

    match state.mode {
        VimMode::Insert => vim_insert_action(key, state),
        VimMode::Normal => vim_normal_action(key, state),
    }
}

fn vim_insert_action(key: KeyEvent, state: &mut VimState) -> Action {
    if key.code == KeyCode::Esc {
        state.mode = VimMode::Normal;
        return Action::EnterNormalMode;
    }
    key_to_action(key)
}

fn vim_normal_action(key: KeyEvent, state: &mut VimState) -> Action {
    // Handle pending keys first
    match state.pending {
        PendingKey::G => {
            state.pending = PendingKey::None;
            return match key.code {
                KeyCode::Char('g') => Action::MoveToFirstLine,
                _ => Action::None,
            };
        }
        PendingKey::D => {
            state.pending = PendingKey::None;
            return match key.code {
                KeyCode::Char('d') => Action::DeleteLine,
                _ => Action::None,
            };
        }
        PendingKey::C => {
            state.pending = PendingKey::None;
            return match key.code {
                KeyCode::Char('c') => {
                    state.mode = VimMode::Insert;
                    Action::ChangeLine
                }
                _ => Action::None,
            };
        }
        PendingKey::None => {}
    }

    match key.code {
        // Mode transitions
        KeyCode::Char('i') => {
            state.mode = VimMode::Insert;
            Action::EnterInsertMode
        }
        KeyCode::Char('a') => {
            state.mode = VimMode::Insert;
            Action::EnterInsertModeAppend
        }
        KeyCode::Char('I') => {
            state.mode = VimMode::Insert;
            Action::EnterInsertModeLineStart
        }
        KeyCode::Char('A') => {
            state.mode = VimMode::Insert;
            Action::EnterInsertModeLineEnd
        }
        KeyCode::Char('o') => {
            state.mode = VimMode::Insert;
            Action::OpenLineBelow
        }
        KeyCode::Char('O') => {
            state.mode = VimMode::Insert;
            Action::OpenLineAbove
        }

        // Motions
        KeyCode::Char('h') | KeyCode::Left => Action::MoveCursorLeft,
        KeyCode::Char('l') | KeyCode::Right => Action::MoveCursorRight,
        KeyCode::Char('j') | KeyCode::Down => Action::ScrollDown,
        KeyCode::Char('k') | KeyCode::Up => Action::ScrollUp,
        KeyCode::Char('w') => Action::MoveCursorWordRight,
        KeyCode::Char('b') => Action::MoveCursorWordLeft,
        KeyCode::Char('e') => Action::MoveCursorWordForwardEnd,
        KeyCode::Char('0') => Action::MoveToLineStart,
        KeyCode::Char('^') => Action::MoveToFirstNonBlank,
        KeyCode::Char('$') => Action::MoveCursorEnd,
        KeyCode::Char('G') => Action::MoveToLastLine,
        KeyCode::Char('g') => {
            state.pending = PendingKey::G;
            Action::None
        }
        KeyCode::Home => Action::MoveCursorHome,
        KeyCode::End => Action::MoveCursorEnd,

        // Editing (dd/cc require double-tap)
        KeyCode::Char('x') => Action::DeleteCharAtCursor,
        KeyCode::Char('d') => {
            state.pending = PendingKey::D;
            Action::None
        }
        KeyCode::Char('c') => {
            state.pending = PendingKey::C;
            Action::None
        }
        KeyCode::Char('u') => Action::Undo,
        KeyCode::Char('p') => Action::PasteClipboard,

        // Quit from normal mode
        KeyCode::Esc => Action::Quit,

        _ => Action::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn key_ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_starts_in_normal_mode() {
        let state = VimState::new();
        assert_eq!(state.mode, VimMode::Normal);
    }

    #[test]
    fn test_i_enters_insert_mode() {
        let mut state = VimState::new();
        let action = vim_key_to_action(key(KeyCode::Char('i')), &mut state);
        assert_eq!(action, Action::EnterInsertMode);
        assert_eq!(state.mode, VimMode::Insert);
    }

    #[test]
    fn test_esc_in_insert_returns_to_normal() {
        let mut state = VimState::new();
        state.mode = VimMode::Insert;
        let action = vim_key_to_action(key(KeyCode::Esc), &mut state);
        assert_eq!(action, Action::EnterNormalMode);
        assert_eq!(state.mode, VimMode::Normal);
    }

    #[test]
    fn test_esc_in_normal_quits() {
        let mut state = VimState::new();
        let action = vim_key_to_action(key(KeyCode::Esc), &mut state);
        assert_eq!(action, Action::Quit);
    }

    #[test]
    fn test_hjkl_motions() {
        let mut state = VimState::new();
        assert_eq!(
            vim_key_to_action(key(KeyCode::Char('h')), &mut state),
            Action::MoveCursorLeft
        );
        assert_eq!(
            vim_key_to_action(key(KeyCode::Char('j')), &mut state),
            Action::ScrollDown
        );
        assert_eq!(
            vim_key_to_action(key(KeyCode::Char('k')), &mut state),
            Action::ScrollUp
        );
        assert_eq!(
            vim_key_to_action(key(KeyCode::Char('l')), &mut state),
            Action::MoveCursorRight
        );
    }

    #[test]
    fn test_word_motions() {
        let mut state = VimState::new();
        assert_eq!(
            vim_key_to_action(key(KeyCode::Char('w')), &mut state),
            Action::MoveCursorWordRight
        );
        assert_eq!(
            vim_key_to_action(key(KeyCode::Char('b')), &mut state),
            Action::MoveCursorWordLeft
        );
        assert_eq!(
            vim_key_to_action(key(KeyCode::Char('e')), &mut state),
            Action::MoveCursorWordForwardEnd
        );
    }

    #[test]
    fn test_gg_goes_to_first_line() {
        let mut state = VimState::new();
        let a1 = vim_key_to_action(key(KeyCode::Char('g')), &mut state);
        assert_eq!(a1, Action::None);
        let a2 = vim_key_to_action(key(KeyCode::Char('g')), &mut state);
        assert_eq!(a2, Action::MoveToFirstLine);
    }

    #[test]
    fn test_g_then_non_g_cancels() {
        let mut state = VimState::new();
        vim_key_to_action(key(KeyCode::Char('g')), &mut state);
        let action = vim_key_to_action(key(KeyCode::Char('x')), &mut state);
        assert_eq!(action, Action::None);
    }

    #[test]
    fn test_dd_deletes_line() {
        let mut state = VimState::new();
        let a1 = vim_key_to_action(key(KeyCode::Char('d')), &mut state);
        assert_eq!(a1, Action::None);
        let a2 = vim_key_to_action(key(KeyCode::Char('d')), &mut state);
        assert_eq!(a2, Action::DeleteLine);
    }

    #[test]
    fn test_d_then_non_d_cancels() {
        let mut state = VimState::new();
        vim_key_to_action(key(KeyCode::Char('d')), &mut state);
        let action = vim_key_to_action(key(KeyCode::Char('j')), &mut state);
        assert_eq!(action, Action::None);
    }

    #[test]
    fn test_cc_changes_line() {
        let mut state = VimState::new();
        let a1 = vim_key_to_action(key(KeyCode::Char('c')), &mut state);
        assert_eq!(a1, Action::None);
        let a2 = vim_key_to_action(key(KeyCode::Char('c')), &mut state);
        assert_eq!(a2, Action::ChangeLine);
        assert_eq!(state.mode, VimMode::Insert);
    }

    #[test]
    fn test_x_deletes_char() {
        let mut state = VimState::new();
        assert_eq!(
            vim_key_to_action(key(KeyCode::Char('x')), &mut state),
            Action::DeleteCharAtCursor
        );
    }

    #[test]
    fn test_global_shortcuts_bypass_vim() {
        let mut state = VimState::new();
        let action = vim_key_to_action(key_ctrl(KeyCode::Char('e')), &mut state);
        assert_eq!(action, Action::SwitchEngine);
        assert_eq!(state.mode, VimMode::Normal);
    }

    #[test]
    fn test_global_shortcut_clears_pending() {
        let mut state = VimState::new();
        vim_key_to_action(key(KeyCode::Char('d')), &mut state);
        let action = vim_key_to_action(key_ctrl(KeyCode::Char('e')), &mut state);
        assert_eq!(action, Action::SwitchEngine);
    }

    #[test]
    fn test_insert_mode_types_chars() {
        let mut state = VimState::new();
        state.mode = VimMode::Insert;
        let action = vim_key_to_action(key(KeyCode::Char('h')), &mut state);
        assert_eq!(action, Action::InsertChar('h'));
    }

    #[test]
    fn test_a_enters_insert_append() {
        let mut state = VimState::new();
        let action = vim_key_to_action(key(KeyCode::Char('a')), &mut state);
        assert_eq!(action, Action::EnterInsertModeAppend);
        assert_eq!(state.mode, VimMode::Insert);
    }

    #[test]
    fn test_tab_bypasses_vim() {
        let mut state = VimState::new();
        let action = vim_key_to_action(key(KeyCode::Tab), &mut state);
        assert_eq!(action, Action::SwitchPanel);
    }

    #[test]
    fn test_u_is_undo_in_normal() {
        let mut state = VimState::new();
        assert_eq!(
            vim_key_to_action(key(KeyCode::Char('u')), &mut state),
            Action::Undo
        );
    }

    #[test]
    fn test_o_opens_line_and_enters_insert() {
        let mut state = VimState::new();
        let action = vim_key_to_action(key(KeyCode::Char('o')), &mut state);
        assert_eq!(action, Action::OpenLineBelow);
        assert_eq!(state.mode, VimMode::Insert);
    }
}
