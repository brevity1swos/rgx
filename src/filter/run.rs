//! TUI event loop for `rgx filter`.

use std::io;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::filter::{FilterApp, Outcome};
use crate::tui::{TerminalGuard, TuiTerminal};

/// Run the TUI event loop to completion. Returns when the user hits Enter
/// (emit) or Esc/q (discard). Input events come from crossterm's blocking
/// `event::read()` — the filter UI has no background work, so async is not
/// needed.
pub fn run_tui(mut app: FilterApp) -> io::Result<(FilterApp, Outcome)> {
    let mut guard = TerminalGuard::enter(false)?;
    let result = event_loop(&mut guard.terminal, &mut app);
    drop(guard);

    result?;
    let outcome = app.outcome;
    Ok((app, outcome))
}

fn event_loop(terminal: &mut TuiTerminal, app: &mut FilterApp) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| crate::filter::ui::render(frame, app))?;
        match crossterm::event::read()? {
            // Windows also reports key releases; act on presses only.
            Event::Key(key) if key.kind == KeyEventKind::Press => handle_key(app, key),
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
        KeyCode::Delete => {
            app.pattern_editor.delete_forward();
            app.recompute();
        }
        KeyCode::Left => app.pattern_editor.move_left(),
        KeyCode::Right => app.pattern_editor.move_right(),
        KeyCode::Home => app.pattern_editor.move_home(),
        KeyCode::End => app.pattern_editor.move_end(),
        KeyCode::Char(c)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            app.pattern_editor.insert_char(c);
            app.recompute();
        }
        _ => {}
    }
}
