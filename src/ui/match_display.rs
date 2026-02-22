use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::engine;
use crate::ui::theme;

pub struct MatchDisplay<'a> {
    pub matches: &'a [engine::Match],
    pub scroll: u16,
    pub focused: bool,
}

impl<'a> Widget for MatchDisplay<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = format!(" Matches ({}) ", self.matches.len());
        let border_color = if self.focused {
            theme::BLUE
        } else {
            theme::OVERLAY
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(title, Style::default().fg(theme::TEXT)));

        if self.matches.is_empty() {
            let paragraph = Paragraph::new(Line::from(Span::styled(
                "No matches",
                Style::default().fg(theme::SUBTEXT),
            )))
            .block(block)
            .style(Style::default().bg(theme::BASE));
            paragraph.render(area, buf);
            return;
        }

        let mut lines = Vec::new();
        for (i, m) in self.matches.iter().enumerate() {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("Match {} ", i + 1),
                    Style::default().fg(theme::BLUE),
                ),
                Span::styled(
                    format!("[{}-{}]: ", m.start, m.end),
                    Style::default().fg(theme::SUBTEXT),
                ),
                Span::styled(
                    format!("\"{}\"", &m.text),
                    Style::default().fg(theme::GREEN),
                ),
            ]));

            for cap in &m.captures {
                let color = theme::capture_color(cap.index.saturating_sub(1));
                let name_str = cap
                    .name
                    .as_ref()
                    .map(|n| format!(" '{n}'"))
                    .unwrap_or_default();
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(
                        format!("Group #{}{name_str} ", cap.index),
                        Style::default().fg(color),
                    ),
                    Span::styled(
                        format!("[{}-{}]: ", cap.start, cap.end),
                        Style::default().fg(theme::SUBTEXT),
                    ),
                    Span::styled(format!("\"{}\"", &cap.text), Style::default().fg(color)),
                ]));
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(block)
            .style(Style::default().bg(theme::BASE))
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        paragraph.render(area, buf);
    }
}
