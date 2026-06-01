use crate::app::App;
use crate::ui;
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_help_key(app: &mut App, key: KeyEvent) {
    let max_scroll = app.help_page_max_scroll();
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            if app.help_scroll_offset < max_scroll {
                app.help_scroll_offset = app.help_scroll_offset.saturating_add(1);
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.help_scroll_offset = app.help_scroll_offset.saturating_sub(1);
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.overlay.help_page = app.overlay.help_page.saturating_sub(1);
            app.help_scroll_offset = 0;
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if app.overlay.help_page + 1 < ui::HELP_PAGE_COUNT {
                app.overlay.help_page += 1;
                app.help_scroll_offset = 0;
            }
        }
        _ => {
            app.overlay.help = false;
            app.overlay.help_page = 0;
            app.help_scroll_offset = 0;
        }
    }
}
