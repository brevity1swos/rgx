use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::explain::ExplainNode;
use crate::ui::theme;

pub struct ExplanationPanel<'a> {
    pub nodes: &'a [ExplainNode],
    pub error: Option<&'a str>,
}

impl<'a> Widget for ExplanationPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY))
            .title(Span::styled(
                " Explanation ",
                Style::default().fg(theme::TEXT),
            ));

        if let Some(err) = self.error {
            let lines = vec![Line::from(Span::styled(
                err.to_string(),
                Style::default().fg(theme::RED),
            ))];
            let paragraph = Paragraph::new(lines)
                .block(block)
                .style(Style::default().bg(theme::BASE));
            paragraph.render(area, buf);
            return;
        }

        if self.nodes.is_empty() {
            let paragraph = Paragraph::new(Line::from(Span::styled(
                "Enter a pattern to see its explanation",
                Style::default().fg(theme::SUBTEXT),
            )))
            .block(block)
            .style(Style::default().bg(theme::BASE));
            paragraph.render(area, buf);
            return;
        }

        let lines: Vec<Line> = self
            .nodes
            .iter()
            .map(|node| {
                let indent = "  ".repeat(node.depth);
                let bullet = if node.depth > 0 { "|- " } else { "" };
                Line::from(Span::styled(
                    format!("{indent}{bullet}{}", node.description),
                    Style::default().fg(if node.depth == 0 {
                        theme::TEXT
                    } else {
                        theme::SUBTEXT
                    }),
                ))
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(block)
            .style(Style::default().bg(theme::BASE))
            .wrap(Wrap { trim: false });

        paragraph.render(area, buf);
    }
}
