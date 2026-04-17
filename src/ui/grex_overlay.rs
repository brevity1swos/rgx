//! grex overlay widget — lets users enter example strings and load a generated regex into the main editor.

use std::time::Instant;

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::grex_integration::GrexOptions;
use crate::input::editor::Editor;
use crate::ui::{centered_overlay, theme};

#[derive(Default)]
pub struct GrexOverlayState {
    pub editor: Editor,
    pub options: GrexOptions,
    pub generated_pattern: Option<String>,
    pub generation_counter: u64,
    pub debounce_deadline: Option<Instant>,
}

pub fn render(frame: &mut Frame, area: Rect, state: &GrexOverlayState) {
    render_with_border(frame, area, state, BorderType::Plain);
}

pub fn render_with_border(frame: &mut Frame, area: Rect, state: &GrexOverlayState, bt: BorderType) {
    let width = area.width.clamp(60, 80);
    let height = area.height.saturating_sub(4).clamp(12, 18);
    let overlay_area = centered_overlay(frame, area, width, height);

    let dim = Style::default().fg(theme::SUBTEXT);
    let title_style = Style::default()
        .fg(theme::BLUE)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(theme::MAUVE);

    let mut lines: Vec<Line<'static>> = Vec::with_capacity(16);
    lines.push(Line::from(Span::styled(
        "Generate Regex from Examples",
        title_style,
    )));
    lines.push(Line::from(""));
    lines.push(build_flag_row(&state.options));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Examples (one per line):",
        label_style,
    )));

    let content = state.editor.content();
    if content.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Enter one example per line. Tab to accept.",
            dim,
        )));
    } else {
        for line in content.lines() {
            lines.push(Line::from(format!("  {line}")));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Generated pattern:", label_style)));
    match state.generated_pattern.as_deref() {
        Some(p) if !p.is_empty() => {
            lines.push(Line::from(Span::styled(
                format!("  {p}"),
                Style::default().fg(theme::GREEN),
            )));
        }
        _ => {
            lines.push(Line::from(Span::styled("  (none yet)", dim)));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Alt+d/a/c: toggle flags    Tab: accept    Esc: cancel",
        dim,
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(bt)
        .border_style(Style::default().fg(theme::BLUE))
        .title(Span::styled(
            " Generate Regex from Examples (Ctrl+X) ",
            Style::default().fg(theme::TEXT),
        ))
        .style(Style::default().bg(theme::BASE));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, overlay_area);
}

fn build_flag_row(options: &GrexOptions) -> Line<'static> {
    fn flag_span(label: &'static str, on: bool) -> Vec<Span<'static>> {
        let marker = if on { "●" } else { "○" };
        let style = if on {
            Style::default().fg(theme::GREEN)
        } else {
            Style::default().fg(theme::SUBTEXT)
        };
        vec![
            Span::raw(label),
            Span::raw(" "),
            Span::styled(marker, style),
        ]
    }
    let mut spans: Vec<Span<'static>> =
        vec![Span::styled("Flags:  ", Style::default().fg(theme::MAUVE))];
    spans.extend(flag_span("[D]igit", options.digit));
    spans.push(Span::raw("   "));
    spans.extend(flag_span("[A]nchors", options.anchors));
    spans.push(Span::raw("   "));
    spans.extend(flag_span("[C]ase-insensitive", options.case_insensitive));
    Line::from(spans)
}
