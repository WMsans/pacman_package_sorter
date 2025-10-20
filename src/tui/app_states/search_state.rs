use crate::tui::app::App;
use crate::tui::app_states::app_state::InputMode;
use crate::tui::app_states::state::KeyEventHandler;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers}; // Import KeyEvent and KeyModifiers
use std::io;

#[derive(Clone, Copy)]
pub struct SearchState;

/// Deletes the word backwards from the current cursor position.
fn delete_word_backward(text: &mut String, cursor_pos: &mut usize) {
    if *cursor_pos == 0 {
        return;
    }
    let original_cursor_pos = *cursor_pos;
    let text_before_cursor = &text[..original_cursor_pos];

    // Find the start of the word to delete
    let new_cursor_pos = text_before_cursor
        .trim_end() // Ignore trailing spaces
        .rfind(' ') // Find the last space
        .map_or(0, |i| i + 1); // If space found, new pos is after it, else 0

    text.drain(new_cursor_pos..original_cursor_pos);
    *cursor_pos = new_cursor_pos;
}

impl KeyEventHandler for SearchState {
    fn handle_key_event(&mut self, app: &mut App, key: KeyEvent) -> io::Result<bool> {
        // Handle modifier keys first
        if key.modifiers == KeyModifiers::CONTROL {
            match key.code {
                // Ctrl + W or Ctrl + Backspace
                KeyCode::Char('w') | KeyCode::Backspace => {
                    delete_word_backward(&mut app.search_input, &mut app.search_cursor_position);
                    app.apply_filters();
                    return Ok(false);
                }
                _ => {}
            }
        }

        // Handle non-modifier keys
        match key.code {
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