//! Terminal session setup/teardown shared by the main TUI and `rgx filter`.
//!
//! The UI draws to stdout when it is a terminal, and to stderr otherwise, so
//! `PATTERN=$(rgx -P)` or `rgx filter | cmd` capture only the real output.
//! [`TerminalGuard`] restores the terminal on drop and on panic, so an error
//! or crash never leaves the shell in raw mode / the alternate screen.

use std::io::{self, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

pub type TuiTerminal = Terminal<CrosstermBackend<Box<dyn Write + Send>>>;

/// Whether a TUI session currently owns the terminal.
static ACTIVE: AtomicBool = AtomicBool::new(false);
static PANIC_HOOK: Once = Once::new();

fn ui_writer() -> Box<dyn Write + Send> {
    if io::stdout().is_terminal() {
        Box::new(io::stdout())
    } else {
        Box::new(io::BufWriter::new(io::stderr()))
    }
}

/// Best-effort restore; a no-op unless a session is active.
fn restore() {
    if !ACTIVE.swap(false, Ordering::SeqCst) {
        return;
    }
    let _ = disable_raw_mode();
    let mut out = ui_writer();
    let _ = execute!(
        out,
        LeaveAlternateScreen,
        DisableMouseCapture,
        crossterm::cursor::Show
    );
}

/// Owns raw mode + the alternate screen for the lifetime of a TUI session.
pub struct TerminalGuard {
    pub terminal: TuiTerminal,
}

impl TerminalGuard {
    pub fn enter(mouse_capture: bool) -> io::Result<Self> {
        PANIC_HOOK.call_once(|| {
            let previous = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                restore();
                previous(info);
            }));
        });

        enable_raw_mode()?;
        ACTIVE.store(true, Ordering::SeqCst);
        let mut out = ui_writer();
        let setup = if mouse_capture {
            execute!(out, EnterAlternateScreen, EnableMouseCapture)
        } else {
            execute!(out, EnterAlternateScreen)
        };
        if let Err(e) = setup {
            restore();
            return Err(e);
        }
        let terminal = match Terminal::new(CrosstermBackend::new(out)) {
            Ok(t) => t,
            Err(e) => {
                restore();
                return Err(e);
            }
        };
        Ok(Self { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = self.terminal.flush();
        restore();
    }
}
