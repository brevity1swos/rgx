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
            let mut style = Style::default().fg(theme::TEXT);
            if absolute == app.selected {
                style = style.add_modifier(Modifier::REVERSED);
            }
            let content = format!("{:>5}  {}", line_idx + 1, app.lines[line_idx]);
            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        format!(" Matches ({}/{}) ", app.matched.len(), app.lines.len()),
        Style::default().fg(theme::BLUE),
    ));
    frame.render_widget(List::new(items).block(block), area);
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
