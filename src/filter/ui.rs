//! Rendering for `rgx filter` TUI mode.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::filter::FilterApp;
use crate::ui::theme;

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
    // Derive the scroll offset so the selected row is always visible. `app.scroll`
    // is retained as a hint for future page-up/down, but the effective start is
    // whatever keeps `selected` in view.
    let start = if inner_height == 0 || app.matched.is_empty() {
        0
    } else {
        (app.selected + 1).saturating_sub(inner_height)
    };
    let end = (start + inner_height).min(app.matched.len());
    let visible = if start < end {
        &app.matched[start..end]
    } else {
        &[][..]
    };
    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(visible_idx, &line_idx)| {
            let absolute = start + visible_idx;
            let is_selected = absolute == app.selected;
            // Empty Vec if invert mode or empty pattern.
            let spans_for_line: &[std::ops::Range<usize>] = app
                .match_spans
                .get(absolute)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            build_line_spans(&app.lines[line_idx], line_idx, spans_for_line, is_selected)
        })
        .collect();
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        format!(" Matches ({}/{}) ", app.matched.len(), app.lines.len()),
        Style::default().fg(theme::BLUE),
    ));
    frame.render_widget(List::new(items).block(block), area);
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
        // Clamp to line length defensively; a malformed range would panic the slice.
        let start = range.start.min(line.len());
        let end = range.end.min(line.len());
        if start < end {
            if start > pos {
                out.push(Span::styled(
                    &line[pos..start],
                    base_style.add_modifier(modifier),
                ));
            }
            let bg = theme::match_bg(i);
            out.push(Span::styled(
                &line[start..end],
                base_style
                    .bg(bg)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(modifier),
            ));
            pos = end;
        }
    }
    if pos < line.len() {
        out.push(Span::styled(
            &line[pos..],
            base_style.add_modifier(modifier),
        ));
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
