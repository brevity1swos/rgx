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
        let flat_spans = build_highlighted_spans(content, self.matches);
        let lines = split_spans_into_lines(flat_spans);

        // Apply vertical scroll
        let v_scroll = self.editor.vertical_scroll();
        let inner_height = (area.height as usize).saturating_sub(2); // borders
        let visible_lines: Vec<Line> = lines
            .into_iter()
            .skip(v_scroll)
            .take(inner_height)
            .collect();

        let paragraph = Paragraph::new(visible_lines)
            .block(block)
            .style(Style::default().bg(theme::BASE));

        paragraph.render(area, buf);

        // Render cursor
        if self.focused {
            let (cursor_line, cursor_col) = self.editor.cursor_line_col();
            let visual_col = cursor_col.saturating_sub(self.editor.scroll_offset());
            let visual_row = cursor_line.saturating_sub(v_scroll);
            let cursor_x = area.x + 1 + visual_col as u16;
            let cursor_y = area.y + 1 + visual_row as u16;
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

/// Split a flat list of spans at newline characters into multiple Lines.
pub fn split_spans_into_lines<'a>(spans: Vec<Span<'a>>) -> Vec<Line<'a>> {
    let mut lines: Vec<Line<'a>> = Vec::new();
    let mut current_spans: Vec<Span<'a>> = Vec::new();

    for span in spans {
        let style = span.style;
        let text: &str = span.content.as_ref();

        let mut remaining = text;
        while let Some(nl_pos) = remaining.find('\n') {
            let before = &remaining[..nl_pos];
            if !before.is_empty() {
                current_spans.push(Span::styled(before.to_string(), style));
            }
            lines.push(Line::from(std::mem::take(&mut current_spans)));
            remaining = &remaining[nl_pos + 1..];
        }
        if !remaining.is_empty() {
            current_spans.push(Span::styled(remaining.to_string(), style));
        }
    }

    // Final line (even if empty — this ensures we always have at least one line)
    lines.push(Line::from(current_spans));
    lines
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
