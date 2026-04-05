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
use rgx::engine::EngineFlags;
use rgx::event::{AppEvent, EventHandler};
use rgx::input::editor::Editor;
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
        use rgx::config::workspace::Workspace;
        let ws = Workspace::load(std::path::Path::new(load_path))?;
        ws.apply(&mut app);
        app.workspace_path = Some(load_path.clone());
    } else if let Some(ref ws_path) = cli.workspace {
        use rgx::config::workspace::Workspace;
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
            use rgx::config::workspace::{print_test_results, Workspace};
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

                    if app.show_benchmark {
                        app.show_benchmark = false;
                        continue;
                    }

                    if app.show_codegen {
                        use crossterm::event::KeyCode;
                        match key.code {
                            KeyCode::Up => {
                                app.codegen_language_index =
                                    app.codegen_language_index.saturating_sub(1);
                            }
                            KeyCode::Down => {
                                let langs = rgx::codegen::Language::all();
                                if app.codegen_language_index + 1 < langs.len() {
                                    app.codegen_language_index += 1;
                                }
                            }
                            KeyCode::Enter => {
                                let langs = rgx::codegen::Language::all();
                                let lang = &langs[app.codegen_language_index];
                                app.generate_code(lang);
                            }
                            _ => {
                                app.show_codegen = false;
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
                            app.focused_panel = (app.focused_panel + 1) % App::PANEL_COUNT;
                        }
                        Action::SwitchPanelBack => {
                            if app.focused_panel == App::PANEL_REGEX {
                                app.commit_pattern_to_history();
                            }
                            app.focused_panel =
                                (app.focused_panel + App::PANEL_COUNT - 1) % App::PANEL_COUNT;
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
                        Action::Benchmark => {
                            app.run_benchmark();
                        }
                        Action::ExportRegex101 => {
                            app.copy_regex101_url();
                        }
                        Action::GenerateCode => {
                            app.show_codegen = true;
                            app.codegen_language_index = 0;
                        }
                        Action::InsertChar(c) => app.edit_focused(|ed| ed.insert_char(c)),
                        Action::InsertNewline => {
                            if app.focused_panel == App::PANEL_TEST {
                                app.test_editor.insert_newline();
                                app.rematch();
                            }
                        }
                        Action::DeleteBack => app.edit_focused(Editor::delete_back),
                        Action::DeleteForward => app.edit_focused(Editor::delete_forward),
                        Action::MoveCursorLeft => app.move_focused(Editor::move_left),
                        Action::MoveCursorRight => app.move_focused(Editor::move_right),
                        Action::MoveCursorWordLeft => app.move_focused(Editor::move_word_left),
                        Action::MoveCursorWordRight => app.move_focused(Editor::move_word_right),
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
                        Action::MoveCursorHome => app.move_focused(Editor::move_home),
                        Action::MoveCursorEnd => app.move_focused(Editor::move_end),
                        Action::DeleteCharAtCursor => {
                            app.edit_focused(Editor::delete_char_at_cursor)
                        }
                        Action::DeleteLine => app.edit_focused(Editor::delete_line),
                        Action::ChangeLine => app.edit_focused(Editor::clear_line),
                        Action::OpenLineBelow => {
                            if app.focused_panel == App::PANEL_TEST {
                                app.test_editor.open_line_below();
                                app.rematch();
                            } else {
                                app.vim_state.cancel_insert();
                            }
                        }
                        Action::OpenLineAbove => {
                            if app.focused_panel == App::PANEL_TEST {
                                app.test_editor.open_line_above();
                                app.rematch();
                            } else {
                                app.vim_state.cancel_insert();
                            }
                        }
                        Action::MoveToFirstNonBlank => {
                            app.move_focused(Editor::move_to_first_non_blank)
                        }
                        Action::MoveToFirstLine => app.move_focused(Editor::move_to_first_line),
                        Action::MoveToLastLine => app.move_focused(Editor::move_to_last_line),
                        Action::MoveCursorWordForwardEnd => {
                            app.move_focused(Editor::move_word_forward_end)
                        }
                        Action::EnterInsertMode => {}
                        Action::EnterInsertModeAppend => app.move_focused(Editor::move_right),
                        Action::EnterInsertModeLineStart => {
                            app.move_focused(Editor::move_to_first_non_blank)
                        }
                        Action::EnterInsertModeLineEnd => app.move_focused(Editor::move_end),
                        Action::EnterNormalMode => app.move_focused(Editor::move_left_in_line),
                        Action::PasteClipboard => {
                            if let Ok(mut cb) = arboard::Clipboard::new() {
                                if let Ok(text) = cb.get_text() {
                                    app.edit_focused(|ed| ed.insert_str(&text));
                                }
                            }
                        }
                        Action::ToggleDebugger => {
                            #[cfg(feature = "pcre2-engine")]
                            if app.debug_session.is_some() {
                                app.close_debug();
                            } else {
                                app.start_debug(settings.debug_max_steps);
                            }
                            #[cfg(not(feature = "pcre2-engine"))]
                            app.start_debug(settings.debug_max_steps);
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
        app.print_output(None, false, false);
    }

    Ok(ExitCode::SUCCESS)
}

fn contains(rect: ratatui::layout::Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}
