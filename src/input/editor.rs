use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone)]
pub struct Editor {
    content: String,
    cursor: usize,
    scroll_offset: usize,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
            scroll_offset: 0,
        }
    }

    pub fn with_content(content: String) -> Self {
        let cursor = content.len();
        Self {
            content,
            cursor,
            scroll_offset: 0,
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

    pub fn visual_cursor(&self) -> usize {
        let before_cursor = &self.content[..self.cursor];
        UnicodeWidthStr::width(before_cursor).saturating_sub(self.scroll_offset)
    }

    pub fn insert_char(&mut self, c: char) {
        self.content.insert(self.cursor, c);
        self.cursor += c.len_utf8();
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

    pub fn move_home(&mut self) {
        self.cursor = 0;
        self.scroll_offset = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.content.len();
    }

    pub fn update_scroll(&mut self, visible_width: usize) {
        let visual = UnicodeWidthStr::width(&self.content[..self.cursor]);
        if visual < self.scroll_offset {
            self.scroll_offset = visual;
        } else if visual >= self.scroll_offset + visible_width {
            self.scroll_offset = visual - visible_width + 1;
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
}
