use std::collections::VecDeque;

use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone)]
pub struct Editor {
    content: String,
    cursor: usize,
    scroll_offset: usize,
    vertical_scroll: usize,
    undo_stack: VecDeque<(String, usize)>,
    redo_stack: VecDeque<(String, usize)>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
            scroll_offset: 0,
            vertical_scroll: 0,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
        }
    }

    pub fn with_content(content: String) -> Self {
        let cursor = content.len();
        Self {
            content,
            cursor,
            scroll_offset: 0,
            vertical_scroll: 0,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn vertical_scroll(&self) -> usize {
        self.vertical_scroll
    }

    /// Returns (line, col) of the cursor where col is the display width within the line.
    pub fn cursor_line_col(&self) -> (usize, usize) {
        let before = &self.content[..self.cursor];
        let line = before.matches('\n').count();
        let line_start = before.rfind('\n').map(|p| p + 1).unwrap_or(0);
        let col = UnicodeWidthStr::width(&self.content[line_start..self.cursor]);
        (line, col)
    }

    pub fn line_count(&self) -> usize {
        self.content.matches('\n').count() + 1
    }

    /// Byte offset of the start of line `n` (0-indexed).
    fn line_start(&self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        let mut count = 0;
        for (i, c) in self.content.char_indices() {
            if c == '\n' {
                count += 1;
                if count == n {
                    return i + 1;
                }
            }
        }
        self.content.len()
    }

    /// Byte offset of the end of line `n` (before the newline, or end of string).
    fn line_end(&self, n: usize) -> usize {
        let start = self.line_start(n);
        match self.content[start..].find('\n') {
            Some(pos) => start + pos,
            None => self.content.len(),
        }
    }

    /// Content of line `n`.
    fn line_content(&self, n: usize) -> &str {
        &self.content[self.line_start(n)..self.line_end(n)]
    }

    /// Visual cursor column within the current line.
    pub fn visual_cursor(&self) -> usize {
        let (_, col) = self.cursor_line_col();
        col.saturating_sub(self.scroll_offset)
    }

    fn push_undo_snapshot(&mut self) {
        self.undo_stack
            .push_back((self.content.clone(), self.cursor));
        if self.undo_stack.len() > 500 {
            self.undo_stack.pop_front();
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> bool {
        if let Some((content, cursor)) = self.undo_stack.pop_back() {
            self.redo_stack
                .push_back((self.content.clone(), self.cursor));
            self.content = content;
            self.cursor = cursor;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some((content, cursor)) = self.redo_stack.pop_back() {
            self.undo_stack
                .push_back((self.content.clone(), self.cursor));
            self.content = content;
            self.cursor = cursor;
            true
        } else {
            false
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.push_undo_snapshot();
        self.content.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn insert_newline(&mut self) {
        self.push_undo_snapshot();
        self.content.insert(self.cursor, '\n');
        self.cursor += 1;
    }

    pub fn delete_back(&mut self) {
        if self.cursor > 0 {
            self.push_undo_snapshot();
            let prev = self.prev_char_boundary();
            self.content.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cursor < self.content.len() {
            self.push_undo_snapshot();
            let next = self.next_char_boundary();
            self.content.drain(self.cursor..next);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.prev_char_boundary();
        }
    }

    /// Move cursor left by one char, but not past the start of the current line.
    /// Used by vim Esc (EnterNormalMode) which should not cross line boundaries.
    pub fn move_left_in_line(&mut self) {
        if self.cursor > 0 && self.content.as_bytes()[self.cursor - 1] != b'\n' {
            self.cursor = self.prev_char_boundary();
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.content.len() {
            self.cursor = self.next_char_boundary();
        }
    }

    /// Move cursor left by one word (to previous word boundary).
    pub fn move_word_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let before = &self.content[..self.cursor];
        let mut chars = before.char_indices().rev();
        // Skip any non-word chars immediately before cursor
        let mut last_idx = self.cursor;
        for (i, c) in &mut chars {
            if c.is_alphanumeric() || c == '_' {
                last_idx = i;
                break;
            }
            last_idx = i;
        }
        // Skip word chars to find the start of the word
        if last_idx < self.cursor {
            let before_word = &self.content[..last_idx];
            for (i, c) in before_word.char_indices().rev() {
                if !(c.is_alphanumeric() || c == '_') {
                    self.cursor = i + c.len_utf8();
                    return;
                }
            }
            // Reached start of string
            self.cursor = 0;
        } else {
            self.cursor = 0;
        }
    }

    /// Move cursor right by one word (to next word boundary).
    pub fn move_word_right(&mut self) {
        if self.cursor >= self.content.len() {
            return;
        }
        let after = &self.content[self.cursor..];
        let mut chars = after.char_indices();
        // Skip any word chars from current position
        let mut advanced = false;
        for (i, c) in &mut chars {
            if !(c.is_alphanumeric() || c == '_') {
                if advanced {
                    self.cursor += i;
                    // Skip non-word chars to reach next word
                    let remaining = &self.content[self.cursor..];
                    for (j, c2) in remaining.char_indices() {
                        if c2.is_alphanumeric() || c2 == '_' {
                            self.cursor += j;
                            return;
                        }
                    }
                    self.cursor = self.content.len();
                    return;
                }
                // We started on non-word chars, skip them
                let remaining = &self.content[self.cursor + i + c.len_utf8()..];
                for (j, c2) in remaining.char_indices() {
                    if c2.is_alphanumeric() || c2 == '_' {
                        self.cursor = self.cursor + i + c.len_utf8() + j;
                        return;
                    }
                }
                self.cursor = self.content.len();
                return;
            }
            advanced = true;
        }
        self.cursor = self.content.len();
    }

    pub fn move_up(&mut self) {
        let (line, col) = self.cursor_line_col();
        if line > 0 {
            let target_line = line - 1;
            let target_start = self.line_start(target_line);
            let target_content = self.line_content(target_line);
            self.cursor = target_start + byte_offset_at_width(target_content, col);
        }
    }

    pub fn move_down(&mut self) {
        let (line, col) = self.cursor_line_col();
        if line + 1 < self.line_count() {
            let target_line = line + 1;
            let target_start = self.line_start(target_line);
            let target_content = self.line_content(target_line);
            self.cursor = target_start + byte_offset_at_width(target_content, col);
        }
    }

    /// Move to start of current line.
    pub fn move_home(&mut self) {
        let (line, _) = self.cursor_line_col();
        self.cursor = self.line_start(line);
        self.scroll_offset = 0;
    }

    /// Move to end of current line.
    pub fn move_end(&mut self) {
        let (line, _) = self.cursor_line_col();
        self.cursor = self.line_end(line);
    }

    /// Delete character under cursor (vim `x`). Does nothing at end of content.
    pub fn delete_char_at_cursor(&mut self) {
        self.delete_forward();
    }

    /// Delete the current line (vim `dd`).
    pub fn delete_line(&mut self) {
        self.push_undo_snapshot();
        let (line, _) = self.cursor_line_col();
        let start = self.line_start(line);
        let end = self.line_end(line);
        let line_count = self.line_count();

        if line_count == 1 {
            self.content.clear();
            self.cursor = 0;
        } else if line + 1 < line_count {
            // Not the last line — delete line including trailing newline
            self.content.drain(start..end + 1);
            self.cursor = start;
        } else {
            // Last line — delete leading newline + line content
            self.content.drain(start - 1..end);
            let prev = line.saturating_sub(1);
            self.cursor = self.line_start(prev);
        }
    }

    /// Clear the current line's content but keep the line (vim `cc`).
    pub fn clear_line(&mut self) {
        self.push_undo_snapshot();
        let (line, _) = self.cursor_line_col();
        let start = self.line_start(line);
        let end = self.line_end(line);
        self.content.drain(start..end);
        self.cursor = start;
    }

    /// Insert a string at cursor (single undo snapshot). Used for paste.
    pub fn insert_str(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        self.push_undo_snapshot();
        self.content.insert_str(self.cursor, s);
        self.cursor += s.len();
    }

    /// Insert a new line below current and move cursor there (vim `o`).
    pub fn open_line_below(&mut self) {
        self.push_undo_snapshot();
        let (line, _) = self.cursor_line_col();
        let end = self.line_end(line);
        self.content.insert(end, '\n');
        self.cursor = end + 1;
    }

    /// Insert a new line above current and move cursor there (vim `O`).
    pub fn open_line_above(&mut self) {
        self.push_undo_snapshot();
        let (line, _) = self.cursor_line_col();
        let start = self.line_start(line);
        self.content.insert(start, '\n');
        self.cursor = start;
    }

    /// Move cursor to first non-whitespace character on current line (vim `^`).
    pub fn move_to_first_non_blank(&mut self) {
        let (line, _) = self.cursor_line_col();
        let start = self.line_start(line);
        let line_text = self.line_content(line);
        let offset = line_text
            .char_indices()
            .find(|(_, c)| !c.is_whitespace())
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.cursor = start + offset;
    }

    /// Move cursor to start of first line (vim `gg`).
    pub fn move_to_first_line(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to start of last line (vim `G`).
    pub fn move_to_last_line(&mut self) {
        let last = self.line_count().saturating_sub(1);
        self.cursor = self.line_start(last);
    }

    /// Move cursor forward to end of current/next word (vim `e`).
    pub fn move_word_forward_end(&mut self) {
        if self.cursor >= self.content.len() {
            return;
        }
        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';
        let after = &self.content[self.cursor..];
        let mut chars = after.char_indices().peekable();

        // Always advance at least one character
        if chars.next().is_none() {
            return;
        }

        // Skip whitespace
        while let Some(&(_, c)) = chars.peek() {
            if !c.is_whitespace() {
                break;
            }
            chars.next();
        }

        // Find end of the word
        if let Some(&(first_offset, first)) = chars.peek() {
            let first_is_word = is_word_char(first);
            let mut last_offset = first_offset;
            chars.next();

            for (i, c) in chars {
                if is_word_char(c) != first_is_word || c.is_whitespace() {
                    break;
                }
                last_offset = i;
            }
            self.cursor += last_offset;
        }
    }

    /// Update horizontal scroll for the current line.
    pub fn update_scroll(&mut self, visible_width: usize) {
        let (_, col) = self.cursor_line_col();
        if col < self.scroll_offset {
            self.scroll_offset = col;
        } else if col >= self.scroll_offset + visible_width {
            self.scroll_offset = col - visible_width + 1;
        }
    }

    /// Update vertical scroll to keep cursor visible within `visible_height` lines.
    pub fn update_vertical_scroll(&mut self, visible_height: usize) {
        let (line, _) = self.cursor_line_col();
        if line < self.vertical_scroll {
            self.vertical_scroll = line;
        } else if line >= self.vertical_scroll + visible_height {
            self.vertical_scroll = line - visible_height + 1;
        }
    }

    /// Set cursor by display column (single-line editors / mouse click).
    pub fn set_cursor_by_col(&mut self, col: usize) {
        self.cursor = byte_offset_at_width(&self.content, col);
    }

    /// Set cursor by (line, col) position (multi-line editors / mouse click).
    pub fn set_cursor_by_position(&mut self, line: usize, col: usize) {
        let target_line = line.min(self.line_count().saturating_sub(1));
        let start = self.line_start(target_line);
        let line_text = self.line_content(target_line);
        self.cursor = start + byte_offset_at_width(line_text, col);
    }

    fn prev_char_boundary(&self) -> usize {
        let mut pos = self.cursor - 1;
        while !self.content.is_char_boundary(pos) {
            pos -= 1;
        }
        pos
    }

    fn next_char_boundary(&self) -> usize {
        let mut pos = self.cursor + 1;
        while pos < self.content.len() && !self.content.is_char_boundary(pos) {
            pos += 1;
        }
        pos
    }
}

/// Convert a target display column width to a byte offset within a line string.
fn byte_offset_at_width(line: &str, target_width: usize) -> usize {
    let mut width = 0;
    for (i, c) in line.char_indices() {
        if width >= target_width {
            return i;
        }
        width += unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
    }
    line.len()
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_content() {
        let mut editor = Editor::new();
        editor.insert_char('h');
        editor.insert_char('i');
        assert_eq!(editor.content(), "hi");
        assert_eq!(editor.cursor(), 2);
    }

    #[test]
    fn test_delete_back() {
        let mut editor = Editor::with_content("hello".to_string());
        editor.delete_back();
        assert_eq!(editor.content(), "hell");
    }

    #[test]
    fn test_cursor_movement() {
        let mut editor = Editor::with_content("hello".to_string());
        editor.move_left();
        assert_eq!(editor.cursor(), 4);
        editor.move_home();
        assert_eq!(editor.cursor(), 0);
        editor.move_end();
        assert_eq!(editor.cursor(), 5);
    }

    #[test]
    fn test_insert_newline() {
        let mut editor = Editor::new();
        editor.insert_char('a');
        editor.insert_newline();
        editor.insert_char('b');
        assert_eq!(editor.content(), "a\nb");
        assert_eq!(editor.cursor(), 3);
    }

    #[test]
    fn test_cursor_line_col() {
        let editor = Editor::with_content("abc\ndef\nghi".to_string());
        // cursor is at end: line 2, col 3
        assert_eq!(editor.cursor_line_col(), (2, 3));
    }

    #[test]
    fn test_move_up_down() {
        let mut editor = Editor::with_content("abc\ndef\nghi".to_string());
        // cursor at end of "ghi" (line 2, col 3)
        editor.move_up();
        assert_eq!(editor.cursor_line_col(), (1, 3));
        assert_eq!(&editor.content()[..editor.cursor()], "abc\ndef");
        editor.move_up();
        assert_eq!(editor.cursor_line_col(), (0, 3));
        assert_eq!(&editor.content()[..editor.cursor()], "abc");
        // move_up at top does nothing
        editor.move_up();
        assert_eq!(editor.cursor_line_col(), (0, 3));
        // move back down
        editor.move_down();
        assert_eq!(editor.cursor_line_col(), (1, 3));
    }

    #[test]
    fn test_move_up_clamps_column() {
        let mut editor = Editor::with_content("abcdef\nab\nxyz".to_string());
        // cursor at end: line 2, col 3
        editor.move_up();
        // line 1 is "ab" (col 2) — should clamp to end of line
        assert_eq!(editor.cursor_line_col(), (1, 2));
        editor.move_up();
        // line 0 is "abcdef" — col 2
        assert_eq!(editor.cursor_line_col(), (0, 2));
    }

    #[test]
    fn test_line_helpers() {
        let editor = Editor::with_content("abc\ndef\nghi".to_string());
        assert_eq!(editor.line_count(), 3);
        assert_eq!(editor.line_content(0), "abc");
        assert_eq!(editor.line_content(1), "def");
        assert_eq!(editor.line_content(2), "ghi");
    }

    #[test]
    fn test_home_end_multiline() {
        let mut editor = Editor::with_content("abc\ndef".to_string());
        // cursor at end of "def" (line 1)
        editor.move_home();
        // should go to start of line 1
        assert_eq!(editor.cursor(), 4); // "abc\n" = 4 bytes
        assert_eq!(editor.cursor_line_col(), (1, 0));
        editor.move_end();
        assert_eq!(editor.cursor(), 7); // "abc\ndef" = 7 bytes
        assert_eq!(editor.cursor_line_col(), (1, 3));
    }

    #[test]
    fn test_vertical_scroll() {
        let mut editor = Editor::with_content("a\nb\nc\nd\ne".to_string());
        editor.update_vertical_scroll(3);
        // cursor at line 4, visible_height 3 => scroll to 2
        assert_eq!(editor.vertical_scroll(), 2);
    }

    #[test]
    fn test_undo_insert() {
        let mut editor = Editor::new();
        editor.insert_char('a');
        editor.insert_char('b');
        assert_eq!(editor.content(), "ab");
        editor.undo();
        assert_eq!(editor.content(), "a");
        editor.undo();
        assert_eq!(editor.content(), "");
        // Undo on empty stack returns false
        assert!(!editor.undo());
    }

    #[test]
    fn test_undo_delete() {
        let mut editor = Editor::with_content("abc".to_string());
        editor.delete_back();
        assert_eq!(editor.content(), "ab");
        editor.undo();
        assert_eq!(editor.content(), "abc");
    }

    #[test]
    fn test_redo() {
        let mut editor = Editor::new();
        editor.insert_char('a');
        editor.insert_char('b');
        editor.undo();
        assert_eq!(editor.content(), "a");
        editor.redo();
        assert_eq!(editor.content(), "ab");
        // Redo on empty stack returns false
        assert!(!editor.redo());
    }

    #[test]
    fn test_redo_cleared_on_new_edit() {
        let mut editor = Editor::new();
        editor.insert_char('a');
        editor.insert_char('b');
        editor.undo();
        // Now type something different — redo stack should clear
        editor.insert_char('c');
        assert_eq!(editor.content(), "ac");
        assert!(!editor.redo());
    }

    #[test]
    fn test_set_cursor_by_col() {
        let mut editor = Editor::with_content("hello".to_string());
        editor.set_cursor_by_col(3);
        assert_eq!(editor.cursor(), 3);
    }

    #[test]
    fn test_set_cursor_by_position() {
        let mut editor = Editor::with_content("abc\ndef\nghi".to_string());
        editor.set_cursor_by_position(1, 2);
        assert_eq!(editor.cursor_line_col(), (1, 2));
    }

    #[test]
    fn test_move_word_right() {
        let mut editor = Editor::with_content("hello world foo".to_string());
        editor.cursor = 0;
        editor.move_word_right();
        // Should skip past "hello" and stop at start of "world"
        assert_eq!(editor.cursor(), 6);
        editor.move_word_right();
        assert_eq!(editor.cursor(), 12);
        editor.move_word_right();
        assert_eq!(editor.cursor(), 15); // end
    }

    #[test]
    fn test_move_word_left() {
        let mut editor = Editor::with_content("hello world foo".to_string());
        // cursor at end
        editor.move_word_left();
        assert_eq!(editor.cursor(), 12); // start of "foo"
        editor.move_word_left();
        assert_eq!(editor.cursor(), 6); // start of "world"
        editor.move_word_left();
        assert_eq!(editor.cursor(), 0); // start of "hello"
    }

    #[test]
    fn test_delete_back_across_newline() {
        let mut editor = Editor::with_content("abc\ndef".to_string());
        // cursor at start of "def" (byte 4)
        editor.cursor = 4;
        editor.delete_back();
        assert_eq!(editor.content(), "abcdef");
        assert_eq!(editor.cursor(), 3);
    }

    #[test]
    fn test_delete_char_at_cursor() {
        let mut editor = Editor::with_content("hello".to_string());
        editor.cursor = 0;
        editor.delete_char_at_cursor();
        assert_eq!(editor.content(), "ello");
        assert_eq!(editor.cursor(), 0);
    }

    #[test]
    fn test_delete_char_at_cursor_end() {
        let mut editor = Editor::with_content("hello".to_string());
        editor.delete_char_at_cursor();
        assert_eq!(editor.content(), "hello");
    }

    #[test]
    fn test_delete_line() {
        let mut editor = Editor::with_content("abc\ndef\nghi".to_string());
        editor.cursor = 5;
        editor.delete_line();
        assert_eq!(editor.content(), "abc\nghi");
        assert_eq!(editor.cursor(), 4);
    }

    #[test]
    fn test_delete_line_last() {
        let mut editor = Editor::with_content("abc\ndef".to_string());
        editor.cursor = 5;
        editor.delete_line();
        assert_eq!(editor.content(), "abc");
        assert_eq!(editor.cursor(), 0);
    }

    #[test]
    fn test_delete_line_single() {
        let mut editor = Editor::with_content("hello".to_string());
        editor.cursor = 2;
        editor.delete_line();
        assert_eq!(editor.content(), "");
        assert_eq!(editor.cursor(), 0);
    }

    #[test]
    fn test_clear_line() {
        let mut editor = Editor::with_content("abc\ndef\nghi".to_string());
        editor.cursor = 5;
        editor.clear_line();
        assert_eq!(editor.content(), "abc\n\nghi");
        assert_eq!(editor.cursor(), 4);
    }

    #[test]
    fn test_clear_line_single() {
        let mut editor = Editor::with_content("hello".to_string());
        editor.cursor = 2;
        editor.clear_line();
        assert_eq!(editor.content(), "");
        assert_eq!(editor.cursor(), 0);
    }

    #[test]
    fn test_insert_str() {
        let mut editor = Editor::with_content("hd".to_string());
        editor.cursor = 1;
        editor.insert_str("ello worl");
        assert_eq!(editor.content(), "hello world");
        assert_eq!(editor.cursor(), 10);
    }

    #[test]
    fn test_insert_str_undo() {
        let mut editor = Editor::with_content("ad".to_string());
        editor.cursor = 1;
        editor.insert_str("bc");
        assert_eq!(editor.content(), "abcd");
        editor.undo();
        assert_eq!(editor.content(), "ad");
    }

    #[test]
    fn test_open_line_below() {
        let mut editor = Editor::with_content("abc\ndef".to_string());
        editor.cursor = 1;
        editor.open_line_below();
        assert_eq!(editor.content(), "abc\n\ndef");
        assert_eq!(editor.cursor(), 4);
    }

    #[test]
    fn test_open_line_above() {
        let mut editor = Editor::with_content("abc\ndef".to_string());
        editor.cursor = 5;
        editor.open_line_above();
        assert_eq!(editor.content(), "abc\n\ndef");
        assert_eq!(editor.cursor(), 4);
    }

    #[test]
    fn test_move_to_first_non_blank() {
        let mut editor = Editor::with_content("   hello".to_string());
        editor.cursor = 7;
        editor.move_to_first_non_blank();
        assert_eq!(editor.cursor(), 3);
    }

    #[test]
    fn test_move_to_first_line() {
        let mut editor = Editor::with_content("abc\ndef\nghi".to_string());
        editor.move_to_first_line();
        assert_eq!(editor.cursor(), 0);
    }

    #[test]
    fn test_move_to_last_line() {
        let mut editor = Editor::with_content("abc\ndef\nghi".to_string());
        editor.cursor = 0;
        editor.move_to_last_line();
        let (line, _) = editor.cursor_line_col();
        assert_eq!(line, 2);
    }

    #[test]
    fn test_move_word_forward_end() {
        let mut editor = Editor::with_content("hello world foo".to_string());
        editor.cursor = 0;
        editor.move_word_forward_end();
        assert_eq!(editor.cursor(), 4);
        editor.move_word_forward_end();
        assert_eq!(editor.cursor(), 10);
    }

    #[test]
    fn test_move_left_in_line_normal() {
        let mut editor = Editor::with_content("hello".to_string());
        // cursor at end (byte 5)
        editor.move_left_in_line();
        assert_eq!(editor.cursor(), 4);
    }

    #[test]
    fn test_move_left_in_line_at_line_start() {
        let mut editor = Editor::with_content("abc\ndef".to_string());
        // cursor at start of "def" (byte 4, right after '\n')
        editor.cursor = 4;
        editor.move_left_in_line();
        // Should NOT cross the newline
        assert_eq!(editor.cursor(), 4);
    }

    #[test]
    fn test_move_left_in_line_at_content_start() {
        let mut editor = Editor::with_content("hello".to_string());
        editor.cursor = 0;
        editor.move_left_in_line();
        assert_eq!(editor.cursor(), 0);
    }
}
