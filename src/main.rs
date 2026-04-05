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
use rgx::config::cli::{Cli, ColorMode};
use rgx::config::settings::Settings;
use rgx::config::workspace::{print_test_results, Workspace};
use rgx::engine::EngineFlags;
use rgx::event::{AppEvent, EventHandler};
use rgx::input::vim::vim_key_to_action;
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

    // Generate shell completions and exit
    if let Some(shell) = cli.completions {
        Cli::print_completions(shell);
        return Ok(ExitCode::SUCCESS);
    }

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
    if cli.rounded || settings.rounded_borders {
        app.rounded_borders = true;
    }
    if cli.vim || settings.vim_mode {
        app.vim_mode = true;
    }

    // Load workspace if --load or --workspace is set
    if let Some(ref load_path) = cli.load {
        let ws = Workspace::load(std::path::Path::new(load_path))?;
        ws.apply(&mut app);
        app.workspace_path = Some(load_path.clone());
    } else if let Some(ref ws_path) = cli.workspace {
        let path = std::path::Path::new(ws_path);
        if path.exists() {
            let ws = Workspace::load(path)?;
            ws.apply(&mut app);
        }
        app.workspace_path = Some(ws_path.clone());
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

    // Test suite mode: --test
    if let Some(test_files) = &cli.test {
        let use_color = io::stdout().is_terminal();
        let mut all_passed = true;
        for path in test_files {
            let ws = match Workspace::load(std::path::Path::new(path)) {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("rgx: {path}: {e}");
                    return Ok(ExitCode::from(2));
                }
            };
            if ws.tests.is_empty() {
                eprintln!("rgx: {path}: no [[tests]] found");
                return Ok(ExitCode::from(2));
            }
            match ws.run_tests() {
                Ok(results) => {
                    if !print_test_results(path, &ws.pattern, &results, use_color) {
                        all_passed = false;
                    }
                }
                Err(e) => {
                    eprintln!("rgx: {path}: {e}");
                    return Ok(ExitCode::from(2));
                }
            }
        }
        return Ok(if all_passed {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        });
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
        if cli.json {
            app.print_json_output();
        } else {
            let use_color = match cli.color {
                ColorMode::Always => true,
                ColorMode::Never => false,
                ColorMode::Auto => io::stdout().is_terminal(),
            };
            app.print_output(cli.group.as_deref(), cli.count, use_color);
        }
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
                    if app.overlay.help {
                        use crossterm::event::KeyCode;
                        match key.code {
                            KeyCode::Left => {
                                app.overlay.help_page = app.overlay.help_page.saturating_sub(1);
                            }
                            KeyCode::Right => {
                                if app.overlay.help_page + 1 < ui::HELP_PAGE_COUNT {
                                    app.overlay.help_page += 1;
                                }
                            }
                            _ => {
                                app.overlay.help = false;
                                app.overlay.help_page = 0;
                            }
                        }
                        continue;
                    }

                    if app.overlay.recipes {
                        use crossterm::event::KeyCode;
                        match key.code {
                            KeyCode::Up => {
                                app.overlay.recipe_index =
                                    app.overlay.recipe_index.saturating_sub(1);
                            }
                            KeyCode::Down => {
                                if app.overlay.recipe_index + 1 < RECIPES.len() {
                                    app.overlay.recipe_index += 1;
                                }
                            }
                            KeyCode::Enter => {
                                let recipe = &RECIPES[app.overlay.recipe_index];
                                app.set_test_string(recipe.test_string);
                                app.set_pattern(recipe.pattern);
                                app.overlay.recipes = false;
                            }
                            _ => {
                                app.overlay.recipes = false;
                            }
                        }
                        continue;
                    }

                    if app.overlay.benchmark {
                        app.overlay.benchmark = false;
                        continue;
                    }

                    if app.overlay.codegen {
                        use crossterm::event::KeyCode;
                        match key.code {
                            KeyCode::Up => {
                                app.overlay.codegen_language_index =
                                    app.overlay.codegen_language_index.saturating_sub(1);
                            }
                            KeyCode::Down => {
                                let langs = rgx::codegen::Language::all();
                                if app.overlay.codegen_language_index + 1 < langs.len() {
                                    app.overlay.codegen_language_index += 1;
                                }
                            }
                            KeyCode::Enter => {
                                let langs = rgx::codegen::Language::all();
                                let lang = &langs[app.overlay.codegen_language_index];
                                app.generate_code(lang);
                            }
                            _ => {
                                app.overlay.codegen = false;
                            }
                        }
                        continue;
                    }

                    #[cfg(feature = "pcre2-engine")]
                    if app.debug_session.is_some() {
                        use crossterm::event::KeyCode;
                        match key.code {
                            KeyCode::Right | KeyCode::Char('l') => app.debug_step_forward(),
                            KeyCode::Left | KeyCode::Char('h') => app.debug_step_back(),
                            KeyCode::Home => app.debug_jump_start(),
                            KeyCode::End | KeyCode::Char('G') => app.debug_jump_end(),
                            KeyCode::Char('g') => app.debug_jump_start(),
                            KeyCode::Char('m') => app.debug_next_match(),
                            KeyCode::Char('f') => app.debug_next_backtrack(),
                            KeyCode::Char('H') => app.debug_toggle_heatmap(),
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.close_debug();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    let action = if app.vim_mode {
                        vim_key_to_action(key, &mut app.vim_state)
                    } else {
                        key_to_action(key)
                    };
                    match action {
                        Action::SaveWorkspace => {
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
                                if let Err(e) = std::fs::create_dir_all(parent) {
                                    app.status.set(format!("Cannot create directory: {e}"));
                                    continue;
                                }
                            }
                            match ws.save(save_path) {
                                Ok(()) => {
                                    app.workspace_path = Some(path.clone());
                                    app.status.set(format!("Saved: {path}"));
                                }
                                Err(e) => {
                                    app.status.set(format!("Save error: {e}"));
                                }
                            }
                        }
                        other => app.handle_action(other, settings.debug_max_steps),
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
                    app.status.tick();
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
        app.print_output(None, false, false);
    }

    Ok(ExitCode::SUCCESS)
}

fn contains(rect: ratatui::layout::Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}
