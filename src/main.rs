use std::io::{self, IsTerminal, Read};
use std::time::Duration;

use clap::Parser;
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
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Event loop
    let mut events = EventHandler::new(Duration::from_millis(50));

    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;

        if let Some(event) = events.next().await {
            match event {
                AppEvent::Key(key) => {
                    if app.show_help {
                        app.show_help = false;
                        continue;
                    }

                    match key_to_action(key) {
                        Action::Quit => {
                            app.should_quit = true;
                        }
                        Action::SwitchPanel => {
                            app.focused_panel = (app.focused_panel + 1) % 5;
                        }
                        Action::SwitchEngine => {
                            app.switch_engine();
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
                                app.scroll_match_up();
                            } else if app.focused_panel == 4 {
                                app.scroll_explain_up();
                            }
                        }
                        Action::ScrollDown => {
                            if app.focused_panel == 1 {
                                app.test_editor.move_down();
                            } else if app.focused_panel == 3 {
                                app.scroll_match_down();
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
                AppEvent::Tick => {}
                AppEvent::Resize(_, _) => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
