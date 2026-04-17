pub mod explanation;
pub mod grex_overlay;
pub mod match_display;
pub mod regex_input;
pub mod replace_input;
pub mod status_bar;
pub mod syntax_highlight;
pub mod test_input;
pub mod theme;

#[cfg(feature = "pcre2-engine")]
pub mod debugger;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, BenchmarkResult};
use crate::codegen;
use crate::engine::EngineKind;
use crate::recipe::RECIPES;
use explanation::ExplanationPanel;
use match_display::MatchDisplay;
use regex_input::RegexInput;
use replace_input::ReplaceInput;
use status_bar::StatusBar;
use test_input::TestInput;

/// Returns the border type based on the rounded_borders flag.
pub(crate) fn border_type(rounded: bool) -> BorderType {
    if rounded {
        BorderType::Rounded
    } else {
        BorderType::Plain
    }
}

/// Panel layout rectangles for mouse hit-testing.
pub struct PanelLayout {
    pub regex_input: Rect,
    pub test_input: Rect,
    pub replace_input: Rect,
    pub match_display: Rect,
    pub explanation: Rect,
    pub status_bar: Rect,
}

pub fn compute_layout(size: Rect) -> PanelLayout {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // regex input
            Constraint::Length(8), // test string input
            Constraint::Length(3), // replacement input
            Constraint::Min(5),    // results area
            Constraint::Length(1), // status bar
        ])
        .split(size);

    let results_chunks = if main_chunks[3].width > 80 {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[3])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[3])
    };

    PanelLayout {
        regex_input: main_chunks[0],
        test_input: main_chunks[1],
        replace_input: main_chunks[2],
        match_display: results_chunks[0],
        explanation: results_chunks[1],
        status_bar: main_chunks[4],
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let layout = compute_layout(size);

    let bt = border_type(app.rounded_borders);

    // Overlays
    if app.overlay.help {
        render_help_overlay(frame, size, app.engine_kind, app.overlay.help_page, bt);
        return;
    }
    if app.overlay.recipes {
        render_recipe_overlay(frame, size, app.overlay.recipe_index, bt);
        return;
    }
    if app.overlay.benchmark {
        render_benchmark_overlay(frame, size, &app.benchmark_results, bt);
        return;
    }
    if app.overlay.codegen {
        render_codegen_overlay(
            frame,
            size,
            app.overlay.codegen_language_index,
            app.regex_editor.content(),
            &app.flags,
            bt,
        );
        return;
    }
    if let Some(grex_state) = app.overlay.grex.as_ref() {
        grex_overlay::render_with_border(frame, size, grex_state, bt);
        return;
    }

    #[cfg(feature = "pcre2-engine")]
    if let Some(ref session) = app.debug_session {
        debugger::render_debugger(frame, size, session, bt);
        return;
    }

    let error_str = app.error.as_deref();

    // Regex input
    frame.render_widget(
        RegexInput {
            editor: &app.regex_editor,
            focused: app.focused_panel == 0,
            error: error_str,
            error_offset: app.error_offset,
            border_type: bt,
            syntax_tokens: &app.syntax_tokens,
        },
        layout.regex_input,
    );

    // Test string input
    frame.render_widget(
        TestInput {
            editor: &app.test_editor,
            focused: app.focused_panel == 1,
            matches: &app.matches,
            show_whitespace: app.show_whitespace,
            border_type: bt,
        },
        layout.test_input,
    );

    // Replacement input
    frame.render_widget(
        ReplaceInput {
            editor: &app.replace_editor,
            focused: app.focused_panel == 2,
            border_type: bt,
        },
        layout.replace_input,
    );

    // Match display
    frame.render_widget(
        MatchDisplay {
            matches: &app.matches,
            replace_result: app.replace_result.as_ref(),
            scroll: app.scroll.match_scroll,
            focused: app.focused_panel == 3,
            selected_match: app.selection.match_index,
            selected_capture: app.selection.capture_index,
            clipboard_status: app.status.text.as_deref(),
            border_type: bt,
        },
        layout.match_display,
    );

    // Explanation panel
    frame.render_widget(
        ExplanationPanel {
            nodes: &app.explanation,
            error: error_str,
            scroll: app.scroll.explain_scroll,
            focused: app.focused_panel == 4,
            border_type: bt,
        },
        layout.explanation,
    );

    // Status bar
    #[cfg(feature = "pcre2-engine")]
    let engine_warning: Option<&'static str> =
        if app.engine_kind == EngineKind::Pcre2 && crate::engine::pcre2::is_pcre2_10_45() {
            Some("CVE-2025-58050: PCRE2 10.45 linked — upgrade to >= 10.46.")
        } else {
            None
        };
    #[cfg(not(feature = "pcre2-engine"))]
    let engine_warning: Option<&'static str> = None;

    frame.render_widget(
        StatusBar {
            engine: app.engine_kind,
            match_count: app.matches.len(),
            flags: app.flags,
            show_whitespace: app.show_whitespace,
            compile_time: app.compile_time,
            match_time: app.match_time,
            vim_mode: if app.vim_mode {
                Some(app.vim_state.mode)
            } else {
                None
            },
            engine_warning,
        },
        layout.status_bar,
    );
}

pub const HELP_PAGE_COUNT: usize = 3;

fn build_help_pages(engine: EngineKind) -> Vec<(String, Vec<Line<'static>>)> {
    let shortcut = |key: &'static str, desc: &'static str| -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("{key:<14}"), Style::default().fg(theme::GREEN)),
            Span::styled(desc, Style::default().fg(theme::TEXT)),
        ])
    };

    // Page 0: Keyboard shortcuts
    let page0 = vec![
        shortcut("Tab/Shift+Tab", "Cycle focus forward/backward"),
        shortcut("Up/Down", "Scroll panel / move cursor / select match"),
        shortcut("Enter", "Insert newline (test string)"),
        shortcut("Ctrl+E", "Cycle regex engine"),
        shortcut("Ctrl+Z", "Undo"),
        shortcut("Ctrl+Shift+Z", "Redo"),
        shortcut("Ctrl+Y", "Copy selected match to clipboard"),
        shortcut("Ctrl+O", "Output results to stdout and quit"),
        shortcut("Ctrl+S", "Save workspace"),
        shortcut("Ctrl+R", "Open regex recipe library"),
        shortcut("Ctrl+B", "Benchmark pattern across all engines"),
        shortcut("Ctrl+U", "Copy regex101.com URL to clipboard"),
        shortcut("Ctrl+D", "Step-through regex debugger"),
        shortcut("Ctrl+G", "Generate code for pattern"),
        shortcut("Ctrl+X", "Generate regex from examples (grex)"),
        shortcut("Ctrl+W", "Toggle whitespace visualization"),
        shortcut("Ctrl+Left/Right", "Move cursor by word"),
        shortcut("Alt+Up/Down", "Browse pattern history"),
        shortcut("Alt+i", "Toggle case-insensitive"),
        shortcut("Alt+m", "Toggle multi-line"),
        shortcut("Alt+s", "Toggle dot-matches-newline"),
        shortcut("Alt+u", "Toggle unicode mode"),
        shortcut("Alt+x", "Toggle extended mode"),
        shortcut("F1", "Show/hide help (Left/Right to page)"),
        shortcut("Esc", "Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Vim: --vim flag | Normal: hjkl wb e 0$^ gg/G x dd cc o/O u p",
            Style::default().fg(theme::SUBTEXT),
        )),
        Line::from(Span::styled(
            "Mouse: click to focus/position, scroll to navigate",
            Style::default().fg(theme::SUBTEXT),
        )),
    ];

    // Page 1: Common regex syntax
    let page1 = vec![
        shortcut(".", "Any character (except newline by default)"),
        shortcut("\\d  \\D", "Digit / non-digit"),
        shortcut("\\w  \\W", "Word char / non-word char"),
        shortcut("\\s  \\S", "Whitespace / non-whitespace"),
        shortcut("\\b  \\B", "Word boundary / non-boundary"),
        shortcut("^  $", "Start / end of line"),
        shortcut("[abc]", "Character class"),
        shortcut("[^abc]", "Negated character class"),
        shortcut("[a-z]", "Character range"),
        shortcut("(group)", "Capturing group"),
        shortcut("(?:group)", "Non-capturing group"),
        shortcut("(?P<n>...)", "Named capturing group"),
        shortcut("a|b", "Alternation (a or b)"),
        shortcut("*  +  ?", "0+, 1+, 0 or 1 (greedy)"),
        shortcut("*?  +?  ??", "Lazy quantifiers"),
        shortcut("{n}  {n,m}", "Exact / range repetition"),
        Line::from(""),
        Line::from(Span::styled(
            "Replacement: $1, ${name}, $0/$&, $$ for literal $",
            Style::default().fg(theme::SUBTEXT),
        )),
    ];

    // Page 2: Engine-specific
    let engine_name = format!("{engine}");
    let page2 = match engine {
        EngineKind::RustRegex => vec![
            Line::from(Span::styled(
                "Rust regex engine — linear time guarantee",
                Style::default().fg(theme::BLUE),
            )),
            Line::from(""),
            shortcut("Unicode", "Full Unicode support by default"),
            shortcut("No lookbehind", "Use fancy-regex or PCRE2 for lookaround"),
            shortcut("No backrefs", "Use fancy-regex or PCRE2 for backrefs"),
            shortcut("\\p{Letter}", "Unicode category"),
            shortcut("(?i)", "Inline case-insensitive flag"),
            shortcut("(?m)", "Inline multi-line flag"),
            shortcut("(?s)", "Inline dot-matches-newline flag"),
            shortcut("(?x)", "Inline extended/verbose flag"),
        ],
        EngineKind::FancyRegex => vec![
            Line::from(Span::styled(
                "fancy-regex engine — lookaround + backreferences",
                Style::default().fg(theme::BLUE),
            )),
            Line::from(""),
            shortcut("(?=...)", "Positive lookahead"),
            shortcut("(?!...)", "Negative lookahead"),
            shortcut("(?<=...)", "Positive lookbehind"),
            shortcut("(?<!...)", "Negative lookbehind"),
            shortcut("\\1  \\2", "Backreferences"),
            shortcut("(?>...)", "Atomic group"),
            Line::from(""),
            Line::from(Span::styled(
                "Delegates to Rust regex for non-fancy patterns",
                Style::default().fg(theme::SUBTEXT),
            )),
        ],
        #[cfg(feature = "pcre2-engine")]
        EngineKind::Pcre2 => vec![
            Line::from(Span::styled(
                "PCRE2 engine — full-featured",
                Style::default().fg(theme::BLUE),
            )),
            Line::from(""),
            shortcut("(?=...)(?!...)", "Lookahead"),
            shortcut("(?<=...)(?<!..)", "Lookbehind"),
            shortcut("\\1  \\2", "Backreferences"),
            shortcut("(?>...)", "Atomic group"),
            shortcut("(*SKIP)(*FAIL)", "Backtracking control verbs"),
            shortcut("(?R)  (?1)", "Recursion / subroutine calls"),
            shortcut("(?(cond)y|n)", "Conditional patterns"),
            shortcut("\\K", "Reset match start"),
            shortcut("(*UTF)", "Force UTF-8 mode"),
        ],
    };

    vec![
        ("Keyboard Shortcuts".to_string(), page0),
        ("Common Regex Syntax".to_string(), page1),
        (format!("Engine: {engine_name}"), page2),
    ]
}

pub(crate) fn centered_overlay(
    frame: &mut Frame,
    area: Rect,
    max_width: u16,
    content_height: u16,
) -> Rect {
    let w = max_width.min(area.width.saturating_sub(4));
    let h = content_height.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let rect = Rect::new(x, y, w, h);
    frame.render_widget(Clear, rect);
    rect
}

fn render_help_overlay(
    frame: &mut Frame,
    area: Rect,
    engine: EngineKind,
    page: usize,
    bt: BorderType,
) {
    let help_area = centered_overlay(frame, area, 64, 24);

    let pages = build_help_pages(engine);
    let current = page.min(pages.len() - 1);
    let (title, content) = &pages[current];

    let mut lines: Vec<Line<'static>> = vec![
        Line::from(Span::styled(
            title.clone(),
            Style::default()
                .fg(theme::BLUE)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    lines.extend(content.iter().cloned());
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            format!(" Page {}/{} ", current + 1, pages.len()),
            Style::default().fg(theme::BASE).bg(theme::BLUE),
        ),
        Span::styled(
            " Left/Right: page | Any other key: close ",
            Style::default().fg(theme::SUBTEXT),
        ),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(bt)
        .border_style(Style::default().fg(theme::BLUE))
        .title(Span::styled(" Help ", Style::default().fg(theme::TEXT)))
        .style(Style::default().bg(theme::BASE));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, help_area);
}

fn render_recipe_overlay(frame: &mut Frame, area: Rect, selected: usize, bt: BorderType) {
    let overlay_area = centered_overlay(frame, area, 70, RECIPES.len() as u16 + 6);

    let mut lines: Vec<Line<'static>> = vec![
        Line::from(Span::styled(
            "Select a recipe to load",
            Style::default()
                .fg(theme::BLUE)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (i, recipe) in RECIPES.iter().enumerate() {
        let is_selected = i == selected;
        let marker = if is_selected { ">" } else { " " };
        let style = if is_selected {
            Style::default().fg(theme::BASE).bg(theme::BLUE)
        } else {
            Style::default().fg(theme::TEXT)
        };
        lines.push(Line::from(Span::styled(
            format!("{marker} {:<24} {}", recipe.name, recipe.description),
            style,
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Up/Down: select | Enter: load | Esc: cancel ",
        Style::default().fg(theme::SUBTEXT),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(bt)
        .border_style(Style::default().fg(theme::GREEN))
        .title(Span::styled(
            " Recipes (Ctrl+R) ",
            Style::default().fg(theme::TEXT),
        ))
        .style(Style::default().bg(theme::BASE));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, overlay_area);
}

fn render_benchmark_overlay(
    frame: &mut Frame,
    area: Rect,
    results: &[BenchmarkResult],
    bt: BorderType,
) {
    let overlay_area = centered_overlay(frame, area, 70, results.len() as u16 + 8);

    let fastest_idx = results
        .iter()
        .enumerate()
        .filter(|(_, r)| r.error.is_none())
        .min_by_key(|(_, r)| r.compile_time + r.match_time)
        .map(|(i, _)| i);

    let mut lines: Vec<Line<'static>> = vec![
        Line::from(Span::styled(
            "Performance Comparison",
            Style::default()
                .fg(theme::BLUE)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!(
                "{:<16} {:>10} {:>10} {:>10} {:>8}",
                "Engine", "Compile", "Match", "Total", "Matches"
            ),
            Style::default()
                .fg(theme::SUBTEXT)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    for (i, result) in results.iter().enumerate() {
        let is_fastest = fastest_idx == Some(i);
        if let Some(ref err) = result.error {
            let line_text = format!("{:<16} {}", result.engine, err);
            lines.push(Line::from(Span::styled(
                line_text,
                Style::default().fg(theme::RED),
            )));
        } else {
            let total = result.compile_time + result.match_time;
            let line_text = format!(
                "{:<16} {:>10} {:>10} {:>10} {:>8}",
                result.engine,
                status_bar::format_duration(result.compile_time),
                status_bar::format_duration(result.match_time),
                status_bar::format_duration(total),
                result.match_count,
            );
            let style = if is_fastest {
                Style::default()
                    .fg(theme::GREEN)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT)
            };
            let mut spans = vec![Span::styled(line_text, style)];
            if is_fastest {
                spans.push(Span::styled(" *", Style::default().fg(theme::GREEN)));
            }
            lines.push(Line::from(spans));
        }
    }

    lines.push(Line::from(""));
    if fastest_idx.is_some() {
        lines.push(Line::from(Span::styled(
            "* = fastest",
            Style::default().fg(theme::GREEN),
        )));
    }
    lines.push(Line::from(Span::styled(
        " Any key: close ",
        Style::default().fg(theme::SUBTEXT),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(bt)
        .border_style(Style::default().fg(theme::PEACH))
        .title(Span::styled(
            " Benchmark (Ctrl+B) ",
            Style::default().fg(theme::TEXT),
        ))
        .style(Style::default().bg(theme::BASE));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, overlay_area);
}

fn render_codegen_overlay(
    frame: &mut Frame,
    area: Rect,
    selected: usize,
    pattern: &str,
    flags: &crate::engine::EngineFlags,
    bt: BorderType,
) {
    let langs = codegen::Language::all();
    let preview = if pattern.is_empty() {
        String::from("(no pattern)")
    } else {
        let lang = &langs[selected.min(langs.len() - 1)];
        codegen::generate_code(lang, pattern, flags)
    };

    let preview_lines: Vec<&str> = preview.lines().collect();
    let preview_height = preview_lines.len() as u16;
    // Languages list + title + spacing + preview + footer
    let content_height = langs.len() as u16 + preview_height + 7;
    let overlay_area = centered_overlay(frame, area, 74, content_height);

    let mut lines: Vec<Line<'static>> = vec![
        Line::from(Span::styled(
            "Select a language to generate code",
            Style::default()
                .fg(theme::MAUVE)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (i, lang) in langs.iter().enumerate() {
        let is_selected = i == selected;
        let marker = if is_selected { ">" } else { " " };
        let style = if is_selected {
            Style::default().fg(theme::BASE).bg(theme::MAUVE)
        } else {
            Style::default().fg(theme::TEXT)
        };
        lines.push(Line::from(Span::styled(format!("{marker} {lang}"), style)));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Preview:",
        Style::default()
            .fg(theme::SUBTEXT)
            .add_modifier(Modifier::BOLD),
    )));
    for pl in preview_lines {
        lines.push(Line::from(Span::styled(
            pl.to_string(),
            Style::default().fg(theme::GREEN),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Up/Down: select | Enter: copy to clipboard | Esc: cancel ",
        Style::default().fg(theme::SUBTEXT),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(bt)
        .border_style(Style::default().fg(theme::MAUVE))
        .title(Span::styled(
            " Code Generation (Ctrl+G) ",
            Style::default().fg(theme::TEXT),
        ))
        .style(Style::default().bg(theme::BASE));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, overlay_area);
}
