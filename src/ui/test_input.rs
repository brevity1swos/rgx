use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::engine;
use crate::input::editor::Editor;
use crate::ui::theme;

pub struct TestInput<'a> {
    pub editor: &'a Editor,
    pub focused: bool,
    pub matches: &'a [engine::Match],
}

impl<'a> Widget for TestInput<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            Style::default().fg(theme::BLUE)
        } else {
            Style::default().fg(theme::OVERLAY)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(Span::styled(
                " Test String ",
                Style::default().fg(theme::TEXT),
            ));

        let content = self.editor.content();
        let spans = build_highlighted_spans(content, self.matches);
        let line = Line::from(spans);

        let paragraph = Paragraph::new(line)
            .block(block)
            .style(Style::default().bg(theme::BASE));

        paragraph.render(area, buf);

        // Render cursor
        if self.focused {
            let cursor_x = area.x + 1 + self.editor.visual_cursor() as u16;
            let cursor_y = area.y + 1;
            if cursor_x < area.x + area.width - 1 && cursor_y < area.y + area.height - 1 {
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

fn build_highlighted_spans<'a>(text: &'a str, matches: &[engine::Match]) -> Vec<Span<'a>> {
    if matches.is_empty() || text.is_empty() {
        return vec![Span::styled(text, Style::default().fg(theme::TEXT))];
    }

    let mut spans = Vec::new();
    let mut pos = 0;

    for m in matches {
        if m.start > pos {
            spans.push(Span::styled(
                &text[pos..m.start],
                Style::default().fg(theme::TEXT),
            ));
        }

        if m.captures.is_empty() {
            spans.push(Span::styled(
                &text[m.start..m.end],
                Style::default()
                    .fg(theme::TEXT)
                    .bg(theme::MATCH_BG)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            // Render with capture group colors
            let mut inner_pos = m.start;
            for cap in &m.captures {
                if cap.start > inner_pos {
                    spans.push(Span::styled(
                        &text[inner_pos..cap.start],
                        Style::default()
                            .fg(theme::TEXT)
                            .bg(theme::MATCH_BG)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
                let color = theme::capture_color(cap.index.saturating_sub(1));
                spans.push(Span::styled(
                    &text[cap.start..cap.end],
                    Style::default()
                        .fg(theme::BASE)
                        .bg(color)
                        .add_modifier(Modifier::BOLD),
                ));
                inner_pos = cap.end;
            }
            if inner_pos < m.end {
                spans.push(Span::styled(
                    &text[inner_pos..m.end],
                    Style::default()
                        .fg(theme::TEXT)
                        .bg(theme::MATCH_BG)
                        .add_modifier(Modifier::BOLD),
                ));
            }
        }

        pos = m.end;
    }

    if pos < text.len() {
        spans.push(Span::styled(&text[pos..], Style::default().fg(theme::TEXT)));
    }

    spans
}
