//! TUI event loop for `rgx filter`.

use std::io;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::filter::{FilterApp, Outcome};

/// Run the TUI event loop to completion. Returns when the user hits Enter
/// (emit) or Esc/q (discard). Input events come from crossterm's blocking
/// `event::read()` — the filter UI has no background work, so async is not
/// needed.
pub fn run_tui(mut app: FilterApp) -> io::Result<(FilterApp, Outcome)> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result?;
    let outcome = app.outcome;
    Ok((app, outcome))
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut FilterApp,
) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| crate::filter::ui::render(frame, app))?;
        match crossterm::event::read()? {
            Event::Key(key) => handle_key(app, key),
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
    Ok(())
}

pub fn handle_key(app: &mut FilterApp, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.outcome = Outcome::Discard;
            app.should_quit = true;
        }
        KeyCode::Char('q') if key.modifiers == KeyModifiers::NONE => {
            app.outcome = Outcome::Discard;
            app.should_quit = true;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.outcome = Outcome::Discard;
            app.should_quit = true;
        }
        KeyCode::Enter => {
            app.outcome = Outcome::Emit;
            app.should_quit = true;
        }
        KeyCode::Up => app.select_prev(),
        KeyCode::Down => app.select_next(),
        KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::ALT) => {
            app.toggle_case_insensitive();
        }
        KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::ALT) => {
            app.toggle_invert();
        }
        KeyCode::Backspace => {
            app.pattern_editor.delete_back();
            app.recompute();
        }
        KeyCode::Left => app.pattern_editor.move_left(),
        KeyCode::Right => app.pattern_editor.move_right(),
        KeyCode::Home => app.pattern_editor.move_home(),
        KeyCode::End => app.pattern_editor.move_end(),
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.pattern_editor.insert_char(c);
            app.recompute();
        }
        _ => {}
    }
}
