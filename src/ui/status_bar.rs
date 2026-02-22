use std::time::Duration;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::engine::{EngineFlags, EngineKind};
use crate::ui::theme;

fn format_duration(d: Duration) -> String {
    let micros = d.as_micros();
    if micros < 1000 {
        format!("{micros}\u{03bc}s")
    } else {
        format!("{:.1}ms", micros as f64 / 1000.0)
    }
}

pub struct StatusBar {
    pub engine: EngineKind,
    pub match_count: usize,
    pub flags: EngineFlags,
    pub show_whitespace: bool,
    pub compile_time: Option<Duration>,
    pub match_time: Option<Duration>,
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

        // Timing info
        if self.compile_time.is_some() || self.match_time.is_some() {
            let mut parts = Vec::new();
            if let Some(ct) = self.compile_time {
                parts.push(format!("compile: {}", format_duration(ct)));
            }
            if let Some(mt) = self.match_time {
                parts.push(format!("match: {}", format_duration(mt)));
            }
            spans.push(Span::styled(
                format!("{} ", parts.join(" | ")),
                Style::default().fg(theme::SUBTEXT).bg(theme::SURFACE0),
            ));
        }

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

        if self.show_whitespace {
            spans.push(Span::styled(
                " \u{00b7} ",
                Style::default()
                    .fg(theme::BASE)
                    .bg(theme::TEAL)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        spans.push(Span::styled(
            " | Tab: switch | Ctrl+E: engine | Ctrl+W: ws | F1: help ",
            Style::default().fg(theme::SUBTEXT).bg(theme::SURFACE0),
        ));

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).style(Style::default().bg(theme::SURFACE0));
        paragraph.render(area, buf);
    }
}
