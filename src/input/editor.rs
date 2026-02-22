use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone)]
pub struct Editor {
    content: String,
    cursor: usize,
    scroll_offset: usize,
    vertical_scroll: usize,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
            scroll_offset: 0,
            vertical_scroll: 0,
        }
    }

    pub fn with_content(content: String) -> Self {
        let cursor = content.len();
        Self {
            content,
            cursor,
            scroll_offset: 0,
            vertical_scroll: 0,
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

    pub fn insert_char(&mut self, c: char) {
        self.content.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn insert_newline(&mut self) {
        self.content.insert(self.cursor, '\n');
        self.cursor += 1;
    }

    pub fn delete_back(&mut self) {
        if self.cursor > 0 {
            let prev = self.prev_char_boundary();
            self.content.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cursor < self.content.len() {
            let next = self.next_char_boundary();
            self.content.drain(self.cursor..next);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.prev_char_boundary();
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.content.len() {
            self.cursor = self.next_char_boundary();
        }
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
    fn test_delete_back_across_newline() {
        let mut editor = Editor::with_content("abc\ndef".to_string());
        // cursor at start of "def" (byte 4)
        editor.cursor = 4;
        editor.delete_back();
        assert_eq!(editor.content(), "abcdef");
        assert_eq!(editor.cursor(), 3);
    }
}
