pub mod explanation;
pub mod match_display;
pub mod regex_input;
pub mod status_bar;
pub mod test_input;
pub mod theme;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::app::App;
use explanation::ExplanationPanel;
use match_display::MatchDisplay;
use regex_input::RegexInput;
use status_bar::StatusBar;
use test_input::TestInput;

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Main layout: inputs on top, results on bottom, status bar at very bottom
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // regex input
            Constraint::Length(8), // test string input (6 visible lines)
            Constraint::Min(5),    // results area
            Constraint::Length(1), // status bar
        ])
        .split(size);

    // Results area: split horizontally between matches and explanation
    let results_chunks = if main_chunks[2].width > 80 {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[2])
    } else {
        // Stack vertically on narrow terminals
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[2])
    };

    // Help overlay
    if app.show_help {
        render_help_overlay(frame, size);
        return;
    }

    let error_str = app.error.as_deref();

    // Regex input
    frame.render_widget(
        RegexInput {
            editor: &app.regex_editor,
            focused: app.focused_panel == 0,
            error: error_str,
        },
        main_chunks[0],
    );

    // Test string input
    frame.render_widget(
        TestInput {
            editor: &app.test_editor,
            focused: app.focused_panel == 1,
            matches: &app.matches,
        },
        main_chunks[1],
    );

    // Match display
    frame.render_widget(
        MatchDisplay {
            matches: &app.matches,
            scroll: app.match_scroll,
            focused: app.focused_panel == 2,
        },
        results_chunks[0],
    );

    // Explanation panel
    frame.render_widget(
        ExplanationPanel {
            nodes: &app.explanation,
            error: error_str,
            scroll: app.explain_scroll,
            focused: app.focused_panel == 3,
        },
        results_chunks[1],
    );

    // Status bar
    frame.render_widget(
        StatusBar {
            engine: app.engine_kind,
            match_count: app.matches.len(),
            flags: app.flags.clone(),
        },
        main_chunks[3],
    );
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    use ratatui::{
        style::Style,
        text::{Line, Span},
        widgets::{Block, Borders, Clear, Paragraph, Wrap},
    };

    let help_width = 60.min(area.width.saturating_sub(4));
    let help_height = 22.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(help_width)) / 2;
    let y = (area.height.saturating_sub(help_height)) / 2;
    let help_area = Rect::new(x, y, help_width, help_height);

    frame.render_widget(Clear, help_area);

    let lines = vec![
        Line::from(Span::styled(
            "rgx - Keyboard Shortcuts",
            Style::default()
                .fg(theme::BLUE)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab       ", Style::default().fg(theme::GREEN)),
            Span::styled(
                "Cycle focus: pattern/test/matches/explanation",
                Style::default().fg(theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("Up/Down   ", Style::default().fg(theme::GREEN)),
            Span::styled(
                "Scroll focused panel / move cursor",
                Style::default().fg(theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("Enter     ", Style::default().fg(theme::GREEN)),
            Span::styled(
                "Insert newline (test string)",
                Style::default().fg(theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+E    ", Style::default().fg(theme::GREEN)),
            Span::styled("Cycle regex engine", Style::default().fg(theme::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("Alt+i     ", Style::default().fg(theme::GREEN)),
            Span::styled("Toggle case-insensitive", Style::default().fg(theme::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("Alt+m     ", Style::default().fg(theme::GREEN)),
            Span::styled("Toggle multi-line", Style::default().fg(theme::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("Alt+s     ", Style::default().fg(theme::GREEN)),
            Span::styled(
                "Toggle dot-matches-newline",
                Style::default().fg(theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("Alt+u     ", Style::default().fg(theme::GREEN)),
            Span::styled("Toggle unicode mode", Style::default().fg(theme::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("Alt+x     ", Style::default().fg(theme::GREEN)),
            Span::styled("Toggle extended mode", Style::default().fg(theme::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("?         ", Style::default().fg(theme::GREEN)),
            Span::styled("Show/hide this help", Style::default().fg(theme::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("Esc       ", Style::default().fg(theme::GREEN)),
            Span::styled("Quit", Style::default().fg(theme::TEXT)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(theme::SUBTEXT),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BLUE))
        .title(Span::styled(" Help ", Style::default().fg(theme::TEXT)))
        .style(Style::default().bg(theme::BASE));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, help_area);
}
