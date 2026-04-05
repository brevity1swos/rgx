use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::input::editor::Editor;
use crate::ui::{syntax_highlight, theme};

pub struct RegexInput<'a> {
    pub editor: &'a Editor,
    pub focused: bool,
    pub error: Option<&'a str>,
    pub error_offset: Option<usize>,
    pub border_type: BorderType,
    pub syntax_tokens: &'a [syntax_highlight::SyntaxToken],
}

impl<'a> Widget for RegexInput<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            Style::default().fg(theme::BLUE)
        } else {
            Style::default().fg(theme::OVERLAY)
        };

        let title = if let Some(err) = self.error {
            let truncated: String = err
                .chars()
                .take((area.width as usize).saturating_sub(10))
                .collect();
            format!(" Pattern (Error: {truncated}) ")
        } else {
            " Pattern ".to_string()
        };

        let title_style = if self.error.is_some() {
            Style::default().fg(theme::RED)
        } else {
            Style::default().fg(theme::TEXT)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(self.border_type)
            .border_style(border_style)
            .title(Span::styled(title, title_style));

        let content = self.editor.content();
        let tokens = self.syntax_tokens;
        let spans = if tokens.is_empty() {
            vec![Span::styled(
                content.to_string(),
                Style::default().fg(theme::TEXT),
            )]
        } else {
            syntax_highlight::build_highlighted_spans(content, tokens)
        };
        let line = Line::from(spans);

        let paragraph = Paragraph::new(line)
            .block(block)
            .style(Style::default().bg(theme::BASE));

        paragraph.render(area, buf);

        // Highlight error character
        if let Some(offset) = self.error_offset {
            let scroll = self.editor.scroll_offset();
            if offset >= scroll {
                let err_x = area.x + 1 + (offset - scroll) as u16;
                let err_y = area.y + 1;
                if err_x < area.x + area.width.saturating_sub(1)
                    && err_y < area.y + area.height.saturating_sub(1)
                {
                    if let Some(cell) = buf.cell_mut((err_x, err_y)) {
                        cell.set_style(Style::default().fg(theme::TEXT).bg(theme::RED));
                    }
                }
            }
        }

        // Render cursor
        if self.focused {
            let cursor_x = area.x + 1 + self.editor.visual_cursor() as u16;
            let cursor_y = area.y + 1;
            if cursor_x < area.x + area.width.saturating_sub(1)
                && cursor_y < area.y + area.height.saturating_sub(1)
            {
                if let Some(cell) = buf.cell_mut((cursor_x, cursor_y)) {
                    cell.set_style(
                        Style::default()
                            .fg(theme::BASE)
                            .bg(theme::TEXT)
                            .add_modifier(Modifier::BOLD),
                    );
                }
            }
        }
    }
}
