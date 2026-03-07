use std::io::{self, IsTerminal, Read};
use std::process::ExitCode;
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
use rgx::config::settings::Settings;
use rgx::engine::EngineFlags;
use rgx::event::{AppEvent, EventHandler};
use rgx::input::editor::Editor;
use rgx::input::{key_to_action, Action};
use rgx::recipe::RECIPES;
use rgx::ui;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(code) => code,
        Err(e) => {
            eprintln!("rgx: {e}");
            ExitCode::from(2)
        }
    }
}

async fn run() -> anyhow::Result<ExitCode> {
    let cli = Cli::parse();
    let settings = Settings::load();

    let engine_kind = match cli.engine {
        Some(ref e) => match e.as_str() {
            "fancy" => rgx::engine::EngineKind::FancyRegex,
            #[cfg(feature = "pcre2-engine")]
            "pcre2" => rgx::engine::EngineKind::Pcre2,
            _ => rgx::engine::EngineKind::RustRegex,
        },
        None => settings.parse_engine(),
    };
    let flags = EngineFlags {
        case_insensitive: cli.case_insensitive || settings.case_insensitive,
        multi_line: cli.multiline || settings.multiline,
        dot_matches_newline: cli.dotall || settings.dotall,
        unicode: cli.unicode.unwrap_or(settings.unicode),
        extended: cli.extended || settings.extended,
    };

    let mut app = App::new(engine_kind, flags);
    if settings.show_whitespace {
        app.show_whitespace = true;
    }

    // Load workspace if --load is set
    if let Some(ref load_path) = cli.load {
        use rgx::config::workspace::Workspace;
        let ws = Workspace::load(std::path::Path::new(load_path))?;
        ws.apply(&mut app);
        app.workspace_path = Some(load_path.clone());
    }

    // Load test string: --text and --file take priority over stdin
    if let Some(text) = &cli.text {
        app.set_test_string(text);
    } else if let Some(path) = &cli.file {
        let text = std::fs::read_to_string(path)?;
        app.set_test_string(&text);
    } else if !io::stdin().is_terminal() {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        app.set_test_string(&buf);
    }

    if let Some(pattern) = &cli.pattern {
        app.set_pattern(pattern);
    }

    if let Some(r) = &cli.replacement {
        app.set_replacement(r);
    }

    // Non-interactive batch mode: --print
    if cli.print {
        if app.regex_editor.content().is_empty() {
            eprintln!("rgx: --print requires a pattern");
            return Ok(ExitCode::from(2));
        }
        if app.test_editor.content().is_empty() {
            eprintln!("rgx: --print requires input (stdin, --file, or --text)");
            return Ok(ExitCode::from(2));
        }
        if let Some(ref err) = app.error {
            eprintln!("rgx: {err}");
            return Ok(ExitCode::from(2));
        }
        app.print_output();
        return Ok(if app.matches.is_empty() {
            ExitCode::from(1)
        } else {
            ExitCode::SUCCESS
        });
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

                    if app.show_recipes {
                        use crossterm::event::KeyCode;
                        match key.code {
                            KeyCode::Up => {
                                app.recipe_index = app.recipe_index.saturating_sub(1);
                            }
                            KeyCode::Down => {
                                if app.recipe_index + 1 < RECIPES.len() {
                                    app.recipe_index += 1;
                                }
                            }
                            KeyCode::Enter => {
                                let recipe = &RECIPES[app.recipe_index];
                                app.set_test_string(recipe.test_string);
                                app.set_pattern(recipe.pattern);
                                app.show_recipes = false;
                            }
                            _ => {
                                app.show_recipes = false;
                            }
                        }
                        continue;
                    }

                    let edit_focused = |app: &mut App, f: fn(&mut Editor)| match app.focused_panel {
                        App::PANEL_REGEX => {
                            f(&mut app.regex_editor);
                            app.recompute();
                        }
                        App::PANEL_TEST => {
                            f(&mut app.test_editor);
                            app.rematch();
                        }
                        App::PANEL_REPLACE => {
                            f(&mut app.replace_editor);
                            app.rereplace();
                        }
                        _ => {}
                    };

                    let move_focused = |app: &mut App, f: fn(&mut Editor)| match app.focused_panel {
                        App::PANEL_REGEX => f(&mut app.regex_editor),
                        App::PANEL_TEST => f(&mut app.test_editor),
                        App::PANEL_REPLACE => f(&mut app.replace_editor),
                        _ => {}
                    };

                    match key_to_action(key) {
                        Action::Quit => {
                            app.should_quit = true;
                        }
                        Action::OutputAndQuit => {
                            app.output_on_quit = true;
                            app.should_quit = true;
                        }
                        Action::SaveWorkspace => {
                            use rgx::config::workspace::Workspace;
                            let ws = Workspace::from_app(&app);
                            let path = app
                                .workspace_path
                                .clone()
                                .or_else(|| {
                                    dirs::config_dir().map(|d| {
                                        d.join("rgx")
                                            .join("workspace.toml")
                                            .to_string_lossy()
                                            .into_owned()
                                    })
                                })
                                .unwrap_or_else(|| "workspace.toml".to_string());
                            let save_path = std::path::Path::new(&path);
                            if let Some(parent) = save_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            match ws.save(save_path) {
                                Ok(()) => {
                                    app.workspace_path = Some(path.clone());
                                    app.set_status_message(format!("Saved: {path}"));
                                }
                                Err(e) => {
                                    app.set_status_message(format!("Save error: {e}"));
                                }
                            }
                        }
                        Action::SwitchPanel => {
                            if app.focused_panel == App::PANEL_REGEX {
                                app.commit_pattern_to_history();
                            }
                            app.focused_panel = (app.focused_panel + 1) % 5;
                        }
                        Action::SwitchEngine => {
                            app.switch_engine();
                        }
                        Action::Undo => {
                            if app.focused_panel == App::PANEL_REGEX && app.regex_editor.undo() {
                                app.recompute();
                            } else if app.focused_panel == App::PANEL_TEST && app.test_editor.undo()
                            {
                                app.rematch();
                            } else if app.focused_panel == App::PANEL_REPLACE
                                && app.replace_editor.undo()
                            {
                                app.rereplace();
                            }
                        }
                        Action::Redo => {
                            if app.focused_panel == App::PANEL_REGEX && app.regex_editor.redo() {
                                app.recompute();
                            } else if app.focused_panel == App::PANEL_TEST && app.test_editor.redo()
                            {
                                app.rematch();
                            } else if app.focused_panel == App::PANEL_REPLACE
                                && app.replace_editor.redo()
                            {
                                app.rereplace();
                            }
                        }
                        Action::HistoryPrev => {
                            if app.focused_panel == App::PANEL_REGEX {
                                app.history_prev();
                            }
                        }
                        Action::HistoryNext => {
                            if app.focused_panel == App::PANEL_REGEX {
                                app.history_next();
                            }
                        }
                        Action::CopyMatch => {
                            if app.focused_panel == App::PANEL_MATCHES {
                                app.copy_selected_match();
                            }
                        }
                        Action::ToggleWhitespace => {
                            app.show_whitespace = !app.show_whitespace;
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
                        Action::OpenRecipes => {
                            app.show_recipes = true;
                            app.recipe_index = 0;
                        }
                        Action::InsertChar(c) => match app.focused_panel {
                            App::PANEL_REGEX => {
                                app.regex_editor.insert_char(c);
                                app.recompute();
                            }
                            App::PANEL_TEST => {
                                app.test_editor.insert_char(c);
                                app.rematch();
                            }
                            App::PANEL_REPLACE => {
                                app.replace_editor.insert_char(c);
                                app.rereplace();
                            }
                            _ => {}
                        },
                        Action::InsertNewline => {
                            if app.focused_panel == App::PANEL_TEST {
                                app.test_editor.insert_newline();
                                app.rematch();
                            }
                        }
                        Action::DeleteBack => edit_focused(&mut app, Editor::delete_back),
                        Action::DeleteForward => edit_focused(&mut app, Editor::delete_forward),
                        Action::MoveCursorLeft => move_focused(&mut app, Editor::move_left),
                        Action::MoveCursorRight => move_focused(&mut app, Editor::move_right),
                        Action::MoveCursorWordLeft => {
                            move_focused(&mut app, Editor::move_word_left)
                        }
                        Action::MoveCursorWordRight => {
                            move_focused(&mut app, Editor::move_word_right)
                        }
                        Action::ScrollUp => match app.focused_panel {
                            App::PANEL_TEST => app.test_editor.move_up(),
                            App::PANEL_MATCHES => app.select_match_prev(),
                            App::PANEL_EXPLAIN => app.scroll_explain_up(),
                            _ => {}
                        },
                        Action::ScrollDown => match app.focused_panel {
                            App::PANEL_TEST => app.test_editor.move_down(),
                            App::PANEL_MATCHES => app.select_match_next(),
                            App::PANEL_EXPLAIN => app.scroll_explain_down(),
                            _ => {}
                        },
                        Action::MoveCursorHome => move_focused(&mut app, Editor::move_home),
                        Action::MoveCursorEnd => move_focused(&mut app, Editor::move_end),
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
                                app.focused_panel = App::PANEL_REGEX;
                                let x = col.saturating_sub(layout.regex_input.x + 1) as usize;
                                app.regex_editor
                                    .set_cursor_by_col(x + app.regex_editor.scroll_offset());
                            } else if contains(layout.test_input, col, row) {
                                app.focused_panel = App::PANEL_TEST;
                                let x = col.saturating_sub(layout.test_input.x + 1) as usize;
                                let y = row.saturating_sub(layout.test_input.y + 1) as usize;
                                let line = y + app.test_editor.vertical_scroll();
                                app.test_editor.set_cursor_by_position(
                                    line,
                                    x + app.test_editor.scroll_offset(),
                                );
                            } else if contains(layout.replace_input, col, row) {
                                app.focused_panel = App::PANEL_REPLACE;
                                let x = col.saturating_sub(layout.replace_input.x + 1) as usize;
                                app.replace_editor
                                    .set_cursor_by_col(x + app.replace_editor.scroll_offset());
                            } else if contains(layout.match_display, col, row) {
                                app.focused_panel = App::PANEL_MATCHES;
                            } else if contains(layout.explanation, col, row) {
                                app.focused_panel = App::PANEL_EXPLAIN;
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
                    app.tick_clipboard_status();
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

    if cli.output_pattern {
        let pattern = app.regex_editor.content();
        if !pattern.is_empty() {
            println!("{pattern}");
        }
    } else if app.output_on_quit {
        app.print_output();
    }

    Ok(ExitCode::SUCCESS)
}

fn contains(rect: ratatui::layout::Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}
