use crate::tui::app::App;
use crate::tui::app_states::app_state::InputMode;
use crate::tui::app_states::state::KeyEventHandler;
use crossterm::event::KeyCode;
use std::io;

#[derive(Clone, Copy)]
pub struct SearchState;

impl KeyEventHandler for SearchState {
    fn handle_key_event(&mut self, app: &mut App, key_code: KeyCode) -> io::Result<bool> {
        match key_code {
            KeyCode::Char(c) => {
                app.search_input.insert(app.search_cursor_position, c);
                app.search_cursor_position += 1;
                app.apply_filters();
            }
            KeyCode::Backspace => {
                if app.search_cursor_position > 0 {
                    app.search_cursor_position -= 1;
                    app.search_input.remove(app.search_cursor_position);
                    app.apply_filters();
                }
            }
            KeyCode::Left => {
                if app.search_cursor_position > 0 {
                    app.search_cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if app.search_cursor_position < app.search_input.len() {
                    app.search_cursor_position += 1;
                }
            }
            KeyCode::Enter => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                app.search_input.clear();
                app.search_cursor_position = 0;
                app.apply_filters();
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(false)
    }
}