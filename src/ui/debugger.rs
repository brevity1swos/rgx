//! Step-through regex debugger overlay.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use super::centered_overlay;
use super::theme;

#[cfg(feature = "pcre2-engine")]
use crate::engine::pcre2_debug::{DebugSession, DebugStep, DebugTrace};

const OVERLAY_WIDTH: u16 = 90;
const OVERLAY_HEIGHT: u16 = 30;

#[cfg(feature = "pcre2-engine")]
pub fn render_debugger(frame: &mut Frame, area: Rect, session: &DebugSession, bt: BorderType) {
    let trace = &session.trace;
    let current_step = session.step;
    let show_heatmap = session.show_heatmap;
    let pattern = session.pattern.as_str();
    let subject = session.subject.as_str();

    let overlay = centered_overlay(frame, area, OVERLAY_WIDTH, OVERLAY_HEIGHT);

    let heatmap_height: u16 = if show_heatmap { 3 } else { 0 };

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(heatmap_height),
            Constraint::Min(3),
            Constraint::Length(2),
        ])
        .split(overlay);

    let border_block = Block::default()
        .borders(Borders::ALL)
        .border_type(bt)
        .border_style(Style::default().fg(theme::RED))
        .title(Span::styled(
            " Step-Through Debugger (Ctrl+D) ",
            Style::default()
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme::BASE));
    frame.render_widget(border_block, overlay);

    if trace.steps.is_empty() {
        let msg = Paragraph::new(Line::from(Span::styled(
            "No steps to display. Enter a pattern and test string, then press Ctrl+D.",
            Style::default().fg(theme::SUBTEXT),
        )))
        .style(Style::default().bg(theme::BASE));
        frame.render_widget(msg, inner_chunks[4]);
        render_controls(frame, inner_chunks[5], show_heatmap);
        return;
    }

    let step = &trace.steps[current_step.min(trace.steps.len() - 1)];

    // The pattern and subject are single lines; scroll each so the current
    // step stays visible. The heatmap shares the pattern's scroll so the two
    // stay aligned.
    let view = inner_chunks[0].width.saturating_sub(2) as usize;
    let token_start = step.pattern_offset;
    let token_end = token_start + step.pattern_item_length.max(1);
    let pattern_scroll = scroll_to_show(
        column_at(pattern, token_start, pattern_char_width),
        column_at(pattern, token_end, pattern_char_width),
        view,
    );
    let pos_col = column_at(subject, step.subject_offset, subject_char_width);
    let subject_scroll = scroll_to_show(pos_col, pos_col + 1, view);

    render_pattern_panel(frame, inner_chunks[0], pattern, step, pattern_scroll, bt);
    render_input_panel(frame, inner_chunks[1], subject, step, subject_scroll, bt);
    render_step_info(frame, inner_chunks[2], step, current_step, trace);

    if show_heatmap {
        render_heatmap(frame, inner_chunks[3], pattern, trace, pattern_scroll, bt);
    }

    render_captures(frame, inner_chunks[4], step, subject, trace, bt);
    render_controls(frame, inner_chunks[5], show_heatmap);
}

fn panel_block(title: &'static str, bt: BorderType) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(bt)
        .border_style(Style::default().fg(theme::OVERLAY))
        .title(Span::styled(title, Style::default().fg(theme::SUBTEXT)))
        .style(Style::default().bg(theme::BASE))
}

/// Drawn width of a pattern character.
#[cfg(feature = "pcre2-engine")]
fn pattern_char_width(c: char) -> usize {
    unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)
}

/// Drawn width of a subject character; whitespace is shown as one-column glyphs.
#[cfg(feature = "pcre2-engine")]
fn subject_char_width(c: char) -> usize {
    match c {
        '\n' | '\t' | ' ' => 1,
        c => unicode_width::UnicodeWidthChar::width(c).unwrap_or(0),
    }
}

/// Column at which the character at byte `pos` is drawn.
#[cfg(feature = "pcre2-engine")]
fn column_at(text: &str, pos: usize, width_of: fn(char) -> usize) -> usize {
    text.char_indices()
        .take_while(|&(i, _)| i < pos)
        .map(|(_, c)| width_of(c))
        .sum()
}

/// Horizontal scroll that keeps columns `start..end` inside a panel `view`
/// columns wide, with about a third of the panel of context before them.
/// `start` is always visible, even when the span is wider than the panel.
#[cfg(feature = "pcre2-engine")]
fn scroll_to_show(start: usize, end: usize, view: usize) -> u16 {
    if view == 0 || end <= view {
        return 0;
    }
    let mut offset = start.saturating_sub(view / 3);
    if end > offset + view {
        offset = (end - view).min(start);
    }
    u16::try_from(offset).unwrap_or(u16::MAX)
}

fn build_char_spans(
    text: &str,
    style_for: impl Fn(usize, char) -> Style,
    display_for: impl Fn(char) -> String,
) -> Vec<Span<'static>> {
    text.char_indices()
        .map(|(i, ch)| Span::styled(display_for(ch), style_for(i, ch)))
        .collect()
}

#[cfg(feature = "pcre2-engine")]
fn render_pattern_panel(
    frame: &mut Frame,
    area: Rect,
    pattern: &str,
    step: &DebugStep,
    scroll: u16,
    bt: BorderType,
) {
    let token_start = step.pattern_offset;
    let token_end = token_start + step.pattern_item_length.max(1);

    let spans = build_char_spans(
        pattern,
        |i, _| {
            if i >= token_start && i < token_end {
                Style::default().fg(theme::BASE).bg(theme::YELLOW)
            } else {
                Style::default().fg(theme::TEXT)
            }
        },
        |ch| ch.to_string(),
    );

    let paragraph = Paragraph::new(Line::from(spans))
        .block(panel_block(" Pattern ", bt))
        .scroll((0, scroll));
    frame.render_widget(paragraph, area);
}

#[cfg(feature = "pcre2-engine")]
fn render_input_panel(
    frame: &mut Frame,
    area: Rect,
    subject: &str,
    step: &DebugStep,
    scroll: u16,
    bt: BorderType,
) {
    let pos = step.subject_offset;

    let mut spans = build_char_spans(
        subject,
        |i, _| {
            if i == pos {
                Style::default().fg(theme::BASE).bg(theme::TEAL)
            } else {
                Style::default().fg(theme::TEXT)
            }
        },
        |ch| match ch {
            '\n' => "↵".to_string(),
            '\t' => "→".to_string(),
            ' ' => "·".to_string(),
            c => c.to_string(),
        },
    );

    // PCRE2 reports position == subject.len() on a trailing match;
    // show a synthetic cursor marker instead of going out-of-bounds.
    if pos >= subject.len() {
        spans.push(Span::styled(
            "⌶",
            Style::default().fg(theme::BASE).bg(theme::TEAL),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans))
        .block(panel_block(" Subject ", bt))
        .scroll((0, scroll));
    frame.render_widget(paragraph, area);
}

#[cfg(feature = "pcre2-engine")]
fn render_step_info(
    frame: &mut Frame,
    area: Rect,
    step: &DebugStep,
    current_step: usize,
    trace: &DebugTrace,
) {
    let total = trace.steps.len();

    let mut spans: Vec<Span<'static>> = vec![
        Span::styled(
            format!("Step {}/{}", current_step + 1, total),
            Style::default()
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
    ];

    if step.is_backtrack {
        spans.push(Span::styled(
            " BACKTRACK ",
            Style::default()
                .fg(theme::BASE)
                .bg(theme::RED)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("  ", Style::default()));
    }

    spans.push(Span::styled(
        format!(
            "Attempt {}/{}",
            step.match_attempt + 1,
            trace.match_attempts
        ),
        Style::default().fg(theme::SUBTEXT),
    ));

    if trace.truncated {
        spans.push(Span::styled("  ", Style::default()));
        spans.push(Span::styled(
            "[TRUNCATED — increase debug_max_steps]",
            Style::default().fg(theme::YELLOW),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme::BASE));
    frame.render_widget(paragraph, area);
}

#[cfg(feature = "pcre2-engine")]
fn render_heatmap(
    frame: &mut Frame,
    area: Rect,
    pattern: &str,
    trace: &DebugTrace,
    scroll: u16,
    bt: BorderType,
) {
    let max_heat = trace.heatmap.iter().copied().max().unwrap_or(1).max(1);

    let mut spans: Vec<Span<'static>> = Vec::new();
    for (i, ch) in pattern.char_indices() {
        let heat = trace
            .byte_to_token
            .get(i)
            .filter(|&&ti| ti != usize::MAX)
            .and_then(|&ti| trace.heatmap.get(ti).copied())
            .unwrap_or(0);

        let pct = heat as f32 / max_heat as f32;
        let bg = if pct < 0.33 {
            theme::BLUE
        } else if pct < 0.66 {
            theme::PEACH
        } else {
            theme::RED
        };

        spans.push(Span::styled(
            ch.to_string(),
            Style::default().fg(theme::BASE).bg(bg),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans))
        .block(panel_block(" Heatmap (H) ", bt))
        .scroll((0, scroll));
    frame.render_widget(paragraph, area);
}

#[cfg(feature = "pcre2-engine")]
fn render_captures(
    frame: &mut Frame,
    area: Rect,
    step: &DebugStep,
    subject: &str,
    trace: &DebugTrace,
    bt: BorderType,
) {
    let mut lines: Vec<Line<'static>> = Vec::new();

    let token_desc = trace
        .byte_to_token
        .get(step.pattern_offset)
        .filter(|&&ti| ti != usize::MAX)
        .and_then(|&ti| trace.offset_map.get(ti))
        .map_or_else(|| "—".to_string(), |t| t.description.clone());

    lines.push(Line::from(vec![
        Span::styled("Token: ", Style::default().fg(theme::SUBTEXT)),
        Span::styled(token_desc, Style::default().fg(theme::YELLOW)),
    ]));

    let captures: Vec<_> = step
        .captures
        .iter()
        .enumerate()
        .filter_map(|(i, c)| c.map(|(s, e)| (i, s, e)))
        .collect();

    if captures.is_empty() {
        lines.push(Line::from(Span::styled(
            "No captures yet",
            Style::default().fg(theme::SUBTEXT),
        )));
    } else {
        for (i, start, end) in captures {
            let text = subject
                .get(start..end)
                .map_or_else(|| "(invalid range)".to_string(), |s| format!("{s:?}"));
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  Group {i}: "),
                    Style::default().fg(theme::SUBTEXT),
                ),
                Span::styled(
                    format!("{text} [{start}..{end}]"),
                    Style::default().fg(theme::GREEN),
                ),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines).block(panel_block(" Token / Captures ", bt));
    frame.render_widget(paragraph, area);
}

fn render_controls(frame: &mut Frame, area: Rect, show_heatmap: bool) {
    let heatmap_label = if show_heatmap {
        "H: hide heatmap"
    } else {
        "H: show heatmap"
    };

    let line1 = Line::from(vec![
        Span::styled("←/h ", Style::default().fg(theme::GREEN)),
        Span::styled("step back  ", Style::default().fg(theme::SUBTEXT)),
        Span::styled("→/l ", Style::default().fg(theme::GREEN)),
        Span::styled("step fwd  ", Style::default().fg(theme::SUBTEXT)),
        Span::styled("Home/g ", Style::default().fg(theme::GREEN)),
        Span::styled("first  ", Style::default().fg(theme::SUBTEXT)),
        Span::styled("End/G ", Style::default().fg(theme::GREEN)),
        Span::styled("last", Style::default().fg(theme::SUBTEXT)),
    ]);

    let line2 = Line::from(vec![
        Span::styled("m ", Style::default().fg(theme::GREEN)),
        Span::styled("next attempt  ", Style::default().fg(theme::SUBTEXT)),
        Span::styled("f ", Style::default().fg(theme::GREEN)),
        Span::styled("next backtrack  ", Style::default().fg(theme::SUBTEXT)),
        Span::styled("H ", Style::default().fg(theme::GREEN)),
        Span::styled(heatmap_label, Style::default().fg(theme::SUBTEXT)),
        Span::styled("  q/Esc ", Style::default().fg(theme::GREEN)),
        Span::styled("close", Style::default().fg(theme::SUBTEXT)),
    ]);

    let paragraph = Paragraph::new(vec![line1, line2]).style(Style::default().bg(theme::BASE));
    frame.render_widget(paragraph, area);
}

#[cfg(all(test, feature = "pcre2-engine"))]
mod tests {
    use super::*;

    #[test]
    fn scroll_to_show_stays_put_when_span_fits() {
        assert_eq!(scroll_to_show(10, 11, 80), 0);
        assert_eq!(scroll_to_show(79, 80, 80), 0);
    }

    #[test]
    fn scroll_to_show_brings_far_span_into_view() {
        let off = scroll_to_show(200, 201, 60) as usize;
        assert!(off <= 200 && 201 <= off + 60);
    }

    #[test]
    fn scroll_to_show_keeps_start_of_oversized_span() {
        assert_eq!(scroll_to_show(100, 300, 60), 100);
    }

    #[test]
    fn column_at_counts_wide_chars() {
        // `é` is one column, `日` is two.
        assert_eq!(column_at("é日x", "é日".len(), pattern_char_width), 3);
        assert_eq!(column_at("a\nb", 2, subject_char_width), 2);
    }
}
