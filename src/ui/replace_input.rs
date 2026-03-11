use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::input::editor::Editor;
use crate::ui::theme;

pub struct ReplaceInput<'a> {
    pub editor: &'a Editor,
    pub focused: bool,
    pub border_type: BorderType,
}

impl<'a> Widget for ReplaceInput<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            Style::default().fg(theme::BLUE)
        } else {
            Style::default().fg(theme::OVERLAY)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(self.border_type)
            .border_style(border_style)
            .title(Span::styled(
                " Replacement ($1, ${name}) ",
                Style::default().fg(theme::TEXT),
            ));

        let content = self.editor.content();
        let line = Line::from(Span::styled(
            content.to_string(),
            Style::default().fg(theme::TEXT),
        ));

        let paragraph = Paragraph::new(line)
            .block(block)
            .style(Style::default().bg(theme::BASE));

        paragraph.render(area, buf);

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
