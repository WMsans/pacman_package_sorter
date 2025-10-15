use crate::tui::app::App;
use crossterm::event::KeyCode;
use std::io;

pub trait KeyEventHandler {
    fn handle_key_event(&mut self, app: &mut App, key_code: KeyCode) -> io::Result<bool>;
}