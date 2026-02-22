use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::engine::{EngineFlags, EngineKind};
use crate::ui::theme;

pub struct StatusBar {
    pub engine: EngineKind,
    pub match_count: usize,
    pub flags: EngineFlags,
}

impl Widget for StatusBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut spans = vec![
            Span::styled(
                format!(" {} ", self.engine),
                Style::default()
                    .fg(theme::BASE)
                    .bg(theme::BLUE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ", Style::default().bg(theme::SURFACE0)),
            Span::styled(
                format!(
                    " {} match{} ",
                    self.match_count,
                    if self.match_count == 1 { "" } else { "es" }
                ),
                Style::default().fg(theme::TEXT).bg(theme::SURFACE0),
            ),
            Span::styled(" ", Style::default().bg(theme::SURFACE0)),
        ];

        // Flag indicators
        let flags = [
            ("i", self.flags.case_insensitive),
            ("m", self.flags.multi_line),
            ("s", self.flags.dot_matches_newline),
            ("u", self.flags.unicode),
            ("x", self.flags.extended),
        ];

        for (name, active) in &flags {
            let style = if *active {
                Style::default()
                    .fg(theme::BASE)
                    .bg(theme::GREEN)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::OVERLAY).bg(theme::SURFACE0)
            };
            spans.push(Span::styled(format!(" {name} "), style));
        }

        spans.push(Span::styled(
            " | Ctrl+E: engine | Tab: switch | F1: help | Esc: quit ",
            Style::default().fg(theme::SUBTEXT).bg(theme::SURFACE0),
        ));

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).style(Style::default().bg(theme::SURFACE0));
        paragraph.render(area, buf);
    }
}
