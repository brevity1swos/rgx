//! Step-through regex debugger overlay.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use super::theme;

#[cfg(feature = "pcre2-engine")]
use crate::engine::pcre2_debug::{find_token_at_offset, DebugStep, DebugTrace};

/// Render the full-screen debugger overlay.
#[cfg(feature = "pcre2-engine")]
#[allow(clippy::too_many_arguments)]
pub fn render_debugger(
    frame: &mut Frame,
    area: Rect,
    trace: &DebugTrace,
    current_step: usize,
    show_heatmap: bool,
    pattern: &str,
    subject: &str,
    bt: BorderType,
) {
    let overlay = centered_overlay(frame, area, 90, 30);

    let heatmap_height: u16 = if show_heatmap { 3 } else { 0 };

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),              // pattern panel
            Constraint::Length(3),              // input panel
            Constraint::Length(2),              // step info
            Constraint::Length(heatmap_height), // heatmap (conditional)
            Constraint::Min(3),                 // captures / description
            Constraint::Length(2),              // controls footer
        ])
        .split(overlay);

    // Outer border
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

    // --- Pattern panel ---
    render_pattern_panel(frame, inner_chunks[0], pattern, step, bt);

    // --- Input panel ---
    render_input_panel(frame, inner_chunks[1], subject, step, bt);

    // --- Step info ---
    render_step_info(frame, inner_chunks[2], step, current_step, trace);

    // --- Heatmap ---
    if show_heatmap {
        render_heatmap(frame, inner_chunks[3], pattern, trace, bt);
    }

    // --- Capture / description ---
    render_captures(frame, inner_chunks[4], step, subject, trace, bt);

    // --- Controls ---
    render_controls(frame, inner_chunks[5], show_heatmap);
}

use super::centered_overlay;

/// Render the pattern with the current token range highlighted in YELLOW.
#[cfg(feature = "pcre2-engine")]
fn render_pattern_panel(
    frame: &mut Frame,
    area: Rect,
    pattern: &str,
    step: &DebugStep,
    bt: BorderType,
) {
    let token_start = step.pattern_offset;
    let token_end = token_start + step.pattern_item_length.max(1);

    let mut spans: Vec<Span<'static>> = Vec::new();
    for (i, ch) in pattern.char_indices() {
        let in_token = i >= token_start && i < token_end;
        let style = if in_token {
            Style::default().fg(theme::BASE).bg(theme::YELLOW)
        } else {
            Style::default().fg(theme::TEXT)
        };
        spans.push(Span::styled(ch.to_string(), style));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(bt)
        .border_style(Style::default().fg(theme::OVERLAY))
        .title(Span::styled(
            " Pattern ",
            Style::default().fg(theme::SUBTEXT),
        ))
        .style(Style::default().bg(theme::BASE));

    let paragraph = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(paragraph, area);
}

/// Render the subject with the current position highlighted in TEAL.
#[cfg(feature = "pcre2-engine")]
fn render_input_panel(
    frame: &mut Frame,
    area: Rect,
    subject: &str,
    step: &DebugStep,
    bt: BorderType,
) {
    let pos = step.subject_offset;

    let mut spans: Vec<Span<'static>> = Vec::new();
    for (i, ch) in subject.char_indices() {
        let at_pos = i == pos;
        let style = if at_pos {
            Style::default().fg(theme::BASE).bg(theme::TEAL)
        } else {
            Style::default().fg(theme::TEXT)
        };
        // Replace control chars with visible representations for display
        let display = match ch {
            '\n' => "↵".to_string(),
            '\t' => "→".to_string(),
            ' ' => "·".to_string(),
            c => c.to_string(),
        };
        spans.push(Span::styled(display, style));
    }

    // If subject_offset is at the end, highlight a synthetic marker
    if pos >= subject.len() {
        spans.push(Span::styled(
            "⌶",
            Style::default().fg(theme::BASE).bg(theme::TEAL),
        ));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(bt)
        .border_style(Style::default().fg(theme::OVERLAY))
        .title(Span::styled(
            " Subject ",
            Style::default().fg(theme::SUBTEXT),
        ))
        .style(Style::default().bg(theme::BASE));

    let paragraph = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(paragraph, area);
}

/// Render step count, backtrack flag, and attempt count.
#[cfg(feature = "pcre2-engine")]
fn render_step_info(
    frame: &mut Frame,
    area: Rect,
    step: &DebugStep,
    current_step: usize,
    trace: &DebugTrace,
) {
    let total = trace.steps.len();
    let attempt_total = trace.match_attempts;

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
        format!("Attempt {}/{}", step.match_attempt + 1, attempt_total),
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

/// Render the heatmap — same pattern text but per-character bg color based on hit count.
#[cfg(feature = "pcre2-engine")]
fn render_heatmap(
    frame: &mut Frame,
    area: Rect,
    pattern: &str,
    trace: &DebugTrace,
    bt: BorderType,
) {
    let max_heat = trace.heatmap.iter().copied().max().unwrap_or(1).max(1);

    let mut spans: Vec<Span<'static>> = Vec::new();
    for (i, ch) in pattern.char_indices() {
        // Find the token index for this character offset
        let heat = if let Some(tok_idx) = find_token_at_offset(&trace.offset_map, i) {
            trace.heatmap.get(tok_idx).copied().unwrap_or(0)
        } else {
            0
        };

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

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(bt)
        .border_style(Style::default().fg(theme::OVERLAY))
        .title(Span::styled(
            " Heatmap (H) ",
            Style::default().fg(theme::SUBTEXT),
        ))
        .style(Style::default().bg(theme::BASE));

    let paragraph = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(paragraph, area);
}

/// Render the capture groups and token description for the current step.
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

    // Token description
    let token_desc = find_token_at_offset(&trace.offset_map, step.pattern_offset)
        .and_then(|idx| trace.offset_map.get(idx))
        .map(|t| t.description.clone())
        .unwrap_or_else(|| "—".to_string());

    lines.push(Line::from(vec![
        Span::styled("Token: ", Style::default().fg(theme::SUBTEXT)),
        Span::styled(token_desc, Style::default().fg(theme::YELLOW)),
    ]));

    // Capture groups
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
                .map(|s| format!("{s:?}"))
                .unwrap_or_else(|| "(invalid range)".to_string());
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

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(bt)
        .border_style(Style::default().fg(theme::OVERLAY))
        .title(Span::styled(
            " Token / Captures ",
            Style::default().fg(theme::SUBTEXT),
        ))
        .style(Style::default().bg(theme::BASE));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

/// Render the controls footer.
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
