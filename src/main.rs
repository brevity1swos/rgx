use std::io::{self, IsTerminal, Read};
use std::time::Duration;

use clap::Parser;
use crossterm::event::{MouseButton, MouseEventKind};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use rgx::app::App;
use rgx::config::cli::Cli;
use rgx::engine::EngineFlags;
use rgx::event::{AppEvent, EventHandler};
use rgx::input::{key_to_action, Action};
use rgx::ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let engine_kind = cli.parse_engine();
    let flags = EngineFlags {
        case_insensitive: cli.case_insensitive,
        multi_line: cli.multiline,
        dot_matches_newline: cli.dotall,
        unicode: cli.unicode,
        extended: cli.extended,
    };

    let mut app = App::new(engine_kind, flags);

    // Read stdin if piped
    let stdin_text = if !io::stdin().is_terminal() {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        Some(buf)
    } else {
        None
    };

    if let Some(text) = &stdin_text {
        app.set_test_string(text);
    }

    if let Some(pattern) = &cli.pattern {
        app.set_pattern(pattern);
    }

    if let Some(r) = &cli.replacement {
        app.set_replacement(r);
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Event loop
    let mut events = EventHandler::new(Duration::from_millis(50));

    let mut last_layout = ui::compute_layout(terminal.get_frame().area());

    loop {
        terminal.draw(|frame| {
            last_layout = ui::compute_layout(frame.area());
            ui::render(frame, &app);
        })?;

        if let Some(event) = events.next().await {
            match event {
                AppEvent::Key(key) => {
                    if app.show_help {
                        use crossterm::event::KeyCode;
                        match key.code {
                            KeyCode::Left => {
                                app.help_page = app.help_page.saturating_sub(1);
                            }
                            KeyCode::Right => {
                                if app.help_page + 1 < ui::HELP_PAGE_COUNT {
                                    app.help_page += 1;
                                }
                            }
                            _ => {
                                app.show_help = false;
                                app.help_page = 0;
                            }
                        }
                        continue;
                    }

                    match key_to_action(key) {
                        Action::Quit => {
                            app.should_quit = true;
                        }
                        Action::SwitchPanel => {
                            if app.focused_panel == 0 {
                                app.commit_pattern_to_history();
                            }
                            app.focused_panel = (app.focused_panel + 1) % 5;
                        }
                        Action::SwitchEngine => {
                            app.switch_engine();
                        }
                        Action::Undo => {
                            if app.focused_panel == 0 && app.regex_editor.undo() {
                                app.recompute();
                            } else if app.focused_panel == 1 && app.test_editor.undo() {
                                app.rematch();
                            } else if app.focused_panel == 2 && app.replace_editor.undo() {
                                app.rereplace();
                            }
                        }
                        Action::Redo => {
                            if app.focused_panel == 0 && app.regex_editor.redo() {
                                app.recompute();
                            } else if app.focused_panel == 1 && app.test_editor.redo() {
                                app.rematch();
                            } else if app.focused_panel == 2 && app.replace_editor.redo() {
                                app.rereplace();
                            }
                        }
                        Action::HistoryPrev => {
                            if app.focused_panel == 0 {
                                app.history_prev();
                            }
                        }
                        Action::HistoryNext => {
                            if app.focused_panel == 0 {
                                app.history_next();
                            }
                        }
                        Action::CopyMatch => {
                            if app.focused_panel == 3 {
                                app.copy_selected_match();
                            }
                        }
                        Action::ToggleCaseInsensitive => {
                            app.flags.toggle_case_insensitive();
                            app.recompute();
                        }
                        Action::ToggleMultiLine => {
                            app.flags.toggle_multi_line();
                            app.recompute();
                        }
                        Action::ToggleDotAll => {
                            app.flags.toggle_dot_matches_newline();
                            app.recompute();
                        }
                        Action::ToggleUnicode => {
                            app.flags.toggle_unicode();
                            app.recompute();
                        }
                        Action::ToggleExtended => {
                            app.flags.toggle_extended();
                            app.recompute();
                        }
                        Action::ShowHelp => {
                            app.show_help = true;
                        }
                        Action::InsertChar(c) => {
                            if app.focused_panel == 0 {
                                app.regex_editor.insert_char(c);
                                app.recompute();
                            } else if app.focused_panel == 1 {
                                app.test_editor.insert_char(c);
                                app.rematch();
                            } else if app.focused_panel == 2 {
                                app.replace_editor.insert_char(c);
                                app.rereplace();
                            }
                        }
                        Action::InsertNewline => {
                            if app.focused_panel == 1 {
                                app.test_editor.insert_newline();
                                app.rematch();
                            }
                        }
                        Action::DeleteBack => {
                            if app.focused_panel == 0 {
                                app.regex_editor.delete_back();
                                app.recompute();
                            } else if app.focused_panel == 1 {
                                app.test_editor.delete_back();
                                app.rematch();
                            } else if app.focused_panel == 2 {
                                app.replace_editor.delete_back();
                                app.rereplace();
                            }
                        }
                        Action::DeleteForward => {
                            if app.focused_panel == 0 {
                                app.regex_editor.delete_forward();
                                app.recompute();
                            } else if app.focused_panel == 1 {
                                app.test_editor.delete_forward();
                                app.rematch();
                            } else if app.focused_panel == 2 {
                                app.replace_editor.delete_forward();
                                app.rereplace();
                            }
                        }
                        Action::MoveCursorLeft => {
                            if app.focused_panel == 0 {
                                app.regex_editor.move_left();
                            } else if app.focused_panel == 1 {
                                app.test_editor.move_left();
                            } else if app.focused_panel == 2 {
                                app.replace_editor.move_left();
                            }
                        }
                        Action::MoveCursorRight => {
                            if app.focused_panel == 0 {
                                app.regex_editor.move_right();
                            } else if app.focused_panel == 1 {
                                app.test_editor.move_right();
                            } else if app.focused_panel == 2 {
                                app.replace_editor.move_right();
                            }
                        }
                        Action::ScrollUp => {
                            if app.focused_panel == 1 {
                                app.test_editor.move_up();
                            } else if app.focused_panel == 3 {
                                app.select_match_prev();
                            } else if app.focused_panel == 4 {
                                app.scroll_explain_up();
                            }
                        }
                        Action::ScrollDown => {
                            if app.focused_panel == 1 {
                                app.test_editor.move_down();
                            } else if app.focused_panel == 3 {
                                app.select_match_next();
                            } else if app.focused_panel == 4 {
                                app.scroll_explain_down();
                            }
                        }
                        Action::MoveCursorHome => {
                            if app.focused_panel == 0 {
                                app.regex_editor.move_home();
                            } else if app.focused_panel == 1 {
                                app.test_editor.move_home();
                            } else if app.focused_panel == 2 {
                                app.replace_editor.move_home();
                            }
                        }
                        Action::MoveCursorEnd => {
                            if app.focused_panel == 0 {
                                app.regex_editor.move_end();
                            } else if app.focused_panel == 1 {
                                app.test_editor.move_end();
                            } else if app.focused_panel == 2 {
                                app.replace_editor.move_end();
                            }
                        }
                        Action::None => {}
                    }
                }
                AppEvent::Mouse(mouse) => {
                    match mouse.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            let col = mouse.column;
                            let row = mouse.row;
                            let layout = &last_layout;

                            // Determine which panel was clicked
                            if contains(layout.regex_input, col, row) {
                                app.focused_panel = 0;
                                let x = col.saturating_sub(layout.regex_input.x + 1) as usize;
                                app.regex_editor
                                    .set_cursor_by_col(x + app.regex_editor.scroll_offset());
                            } else if contains(layout.test_input, col, row) {
                                app.focused_panel = 1;
                                let x = col.saturating_sub(layout.test_input.x + 1) as usize;
                                let y = row.saturating_sub(layout.test_input.y + 1) as usize;
                                let line = y + app.test_editor.vertical_scroll();
                                app.test_editor.set_cursor_by_position(
                                    line,
                                    x + app.test_editor.scroll_offset(),
                                );
                            } else if contains(layout.replace_input, col, row) {
                                app.focused_panel = 2;
                                let x = col.saturating_sub(layout.replace_input.x + 1) as usize;
                                app.replace_editor
                                    .set_cursor_by_col(x + app.replace_editor.scroll_offset());
                            } else if contains(layout.match_display, col, row) {
                                app.focused_panel = 3;
                            } else if contains(layout.explanation, col, row) {
                                app.focused_panel = 4;
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            let col = mouse.column;
                            let row = mouse.row;
                            let layout = &last_layout;

                            if contains(layout.test_input, col, row) {
                                app.test_editor.move_up();
                            } else if contains(layout.match_display, col, row) {
                                app.select_match_prev();
                            } else if contains(layout.explanation, col, row) {
                                app.scroll_explain_up();
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            let col = mouse.column;
                            let row = mouse.row;
                            let layout = &last_layout;

                            if contains(layout.test_input, col, row) {
                                app.test_editor.move_down();
                            } else if contains(layout.match_display, col, row) {
                                app.select_match_next();
                            } else if contains(layout.explanation, col, row) {
                                app.scroll_explain_down();
                            }
                        }
                        _ => {}
                    }
                }
                AppEvent::Tick => {
                    // Auto-dismiss clipboard status
                    if app.clipboard_status.is_some() {
                        app.clipboard_status = None;
                    }
                }
                AppEvent::Resize(_, _) => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn contains(rect: ratatui::layout::Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}
