//! Rendering for `rgx filter` TUI mode.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::filter::FilterApp;
use crate::ui::theme;

/// Slice a string by a byte range, returning `""` on any invalid or
/// char-boundary-crossing range. The `regex` crate always returns char-aligned
/// offsets in practice, so this only matters for defensively-handled spans
/// constructed by callers — we never want to panic the TUI on an odd input.
fn safe_slice(s: &str, start: usize, end: usize) -> &str {
    s.get(start..end).unwrap_or("")
}

pub fn render(frame: &mut Frame, app: &FilterApp) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    render_pattern_pane(frame, chunks[0], app);
    render_match_list(frame, chunks[1], app);
    render_status(frame, chunks[2], app);
}

fn render_pattern_pane(frame: &mut Frame, area: Rect, app: &FilterApp) {
    let content = app.pattern();
    let style = if app.error.is_some() {
        Style::default().fg(theme::RED)
    } else {
        Style::default().fg(theme::TEXT)
    };
    let title = if app.error.is_some() {
        " Pattern (invalid) "
    } else {
        " Pattern "
    };
    let block = Block::default()
        .title(Span::styled(title, Style::default().fg(theme::BLUE)))
        .borders(Borders::ALL);
    let paragraph =
        Paragraph::new(Line::from(Span::styled(content.to_string(), style))).block(block);
    frame.render_widget(paragraph, area);
}

fn render_match_list(frame: &mut Frame, area: Rect, app: &FilterApp) {
    if let Some(err) = app.error.as_deref() {
        let block = Block::default().borders(Borders::ALL).title(" Matches ");
        let paragraph = Paragraph::new(Line::from(Span::styled(
            format!("error: {err}"),
            Style::default().fg(theme::RED),
        )))
        .block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let inner_height = area.height.saturating_sub(2) as usize;
    // --json mode renders two lines per row (raw JSON + extracted value).
    // Narrow terminals (< 60 cols) fall back to single-line rendering to keep
    // the display legible.
    let two_line = app.json_extracted.is_some() && area.width >= 60;
    let rows_per_entry = if two_line { 2 } else { 1 };
    let max_rows = inner_height / rows_per_entry;

    // Derive the scroll offset so the selected row is always visible. `app.scroll`
    // is retained as a hint for future page-up/down, but the effective start is
    // whatever keeps `selected` in view.
    let start = if max_rows == 0 || app.matched.is_empty() {
        0
    } else {
        (app.selected + 1).saturating_sub(max_rows)
    };
    let end = (start + max_rows).min(app.matched.len());
    let visible = if start < end {
        &app.matched[start..end]
    } else {
        &[][..]
    };
    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .flat_map(|(visible_idx, &line_idx)| {
            let absolute = start + visible_idx;
            let is_selected = absolute == app.selected;
            // Empty Vec if invert mode or empty pattern.
            let spans_for_line: &[std::ops::Range<usize>] = app
                .match_spans
                .get(absolute)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            build_row(app, line_idx, spans_for_line, is_selected, two_line)
        })
        .collect();
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        format!(" Matches ({}/{}) ", app.matched.len(), app.lines.len()),
        Style::default().fg(theme::BLUE),
    ));
    frame.render_widget(List::new(items).block(block), area);
}

/// Build one or two `ListItem`s for a single matched line.
///
/// Returns two items when `two_line` is `true` (--json mode on a wide enough
/// terminal): first the raw JSON line dimmed, then `↳ <extracted>` with the
/// match spans highlighted. Otherwise returns a single item whose content
/// depends on whether --json is active: either the raw line with spans, or
/// the extracted value with spans (narrow --json fallback).
fn build_row<'a>(
    app: &'a FilterApp,
    line_idx: usize,
    spans: &[std::ops::Range<usize>],
    is_selected: bool,
    two_line: bool,
) -> Vec<ListItem<'a>> {
    let raw = &app.lines[line_idx];
    let extracted = app
        .json_extracted
        .as_ref()
        .and_then(|v| v.get(line_idx).and_then(|o| o.as_deref()));

    match (extracted, two_line) {
        (Some(ext), true) => {
            // Two-line row: raw JSON (dim, no span highlights) + extracted with spans.
            let raw_item = build_raw_context(raw, line_idx, is_selected);
            let ext_item = build_extracted(ext, spans, is_selected);
            vec![raw_item, ext_item]
        }
        (Some(ext), false) => {
            // Narrow fallback: render only the extracted value with spans.
            // We still prefix with the line number so selection/orientation is clear.
            vec![build_line_spans(ext, line_idx, spans, is_selected)]
        }
        (None, _) => {
            // No --json: render the raw line with match spans (existing behavior).
            vec![build_line_spans(raw, line_idx, spans, is_selected)]
        }
    }
}

fn build_raw_context(line: &str, line_idx: usize, is_selected: bool) -> ListItem<'_> {
    let modifier = if is_selected {
        Modifier::REVERSED | Modifier::DIM
    } else {
        Modifier::DIM
    };
    let style = Style::default().fg(theme::SUBTEXT).add_modifier(modifier);
    let prefix = Span::styled(format!("{:>5}  ", line_idx + 1), style);
    let body = Span::styled(line, style);
    ListItem::new(Line::from(vec![prefix, body]))
}

fn build_extracted<'a>(
    extracted: &'a str,
    spans: &[std::ops::Range<usize>],
    is_selected: bool,
) -> ListItem<'a> {
    let base_style = Style::default().fg(theme::TEXT);
    let modifier = if is_selected {
        Modifier::REVERSED
    } else {
        Modifier::empty()
    };

    let mut out: Vec<Span<'a>> = Vec::new();
    // 7-char indent to align with the line-number prefix width above (5) + 2 spaces.
    out.push(Span::styled(
        "     \u{21b3} ",
        Style::default().fg(theme::BLUE).add_modifier(modifier),
    ));

    if spans.is_empty() {
        out.push(Span::styled(extracted, base_style.add_modifier(modifier)));
        return ListItem::new(Line::from(out));
    }

    let mut pos = 0;
    for (i, range) in spans.iter().enumerate() {
        let start = range.start.min(extracted.len());
        let end = range.end.min(extracted.len());
        if start < end {
            if start > pos {
                let chunk = safe_slice(extracted, pos, start);
                if !chunk.is_empty() {
                    out.push(Span::styled(chunk, base_style.add_modifier(modifier)));
                }
            }
            let bg = theme::match_bg(i);
            let chunk = safe_slice(extracted, start, end);
            if !chunk.is_empty() {
                out.push(Span::styled(
                    chunk,
                    base_style
                        .bg(bg)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(modifier),
                ));
            }
            pos = end;
        }
    }
    if pos < extracted.len() {
        let chunk = safe_slice(extracted, pos, extracted.len());
        if !chunk.is_empty() {
            out.push(Span::styled(chunk, base_style.add_modifier(modifier)));
        }
    }

    ListItem::new(Line::from(out))
}

/// Build a styled `ListItem` for a single line, alternating match-span backgrounds
/// to match the main rgx match-display panel. When `is_selected` is true the
/// entire row is reversed to preserve the existing selection indicator.
fn build_line_spans<'a>(
    line: &'a str,
    line_idx: usize,
    spans: &[std::ops::Range<usize>],
    is_selected: bool,
) -> ListItem<'a> {
    let base_style = Style::default().fg(theme::TEXT);
    let modifier = if is_selected {
        Modifier::REVERSED
    } else {
        Modifier::empty()
    };

    let mut out: Vec<Span<'a>> = Vec::new();
    // Prefix stays unstyled (no match background) but still reverses on selection.
    out.push(Span::styled(
        format!("{:>5}  ", line_idx + 1),
        base_style.add_modifier(modifier),
    ));

    if spans.is_empty() {
        out.push(Span::styled(line, base_style.add_modifier(modifier)));
        return ListItem::new(Line::from(out));
    }

    let mut pos = 0;
    for (i, range) in spans.iter().enumerate() {
        // Clamp to line length so `pos = end` below stays inside the string —
        // otherwise the trailing "emit remainder" branch would skip content
        // when `end > line.len()`. `safe_slice` already handles the slice
        // itself.
        let start = range.start.min(line.len());
        let end = range.end.min(line.len());
        if start < end {
            if start > pos {
                let chunk = safe_slice(line, pos, start);
                if !chunk.is_empty() {
                    out.push(Span::styled(chunk, base_style.add_modifier(modifier)));
                }
            }
            let bg = theme::match_bg(i);
            let chunk = safe_slice(line, start, end);
            if !chunk.is_empty() {
                out.push(Span::styled(
                    chunk,
                    base_style
                        .bg(bg)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(modifier),
                ));
            }
            pos = end;
        }
    }
    if pos < line.len() {
        let chunk = safe_slice(line, pos, line.len());
        if !chunk.is_empty() {
            out.push(Span::styled(chunk, base_style.add_modifier(modifier)));
        }
    }

    ListItem::new(Line::from(out))
}

fn render_status(frame: &mut Frame, area: Rect, app: &FilterApp) {
    let flags = if app.options.case_insensitive {
        "i"
    } else {
        "-"
    };
    let invert = if app.options.invert { "v" } else { "-" };
    let text = format!(
        " flags: [{flags}{invert}]   Enter: emit  Esc: discard  Alt+i: case  Alt+v: invert  Up/Down: browse "
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(theme::SUBTEXT),
        ))),
        area,
    );
}
