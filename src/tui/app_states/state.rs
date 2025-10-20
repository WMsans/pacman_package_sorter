use crate::tui::app::App;
use crossterm::event::KeyEvent; 
use std::io;

pub trait KeyEventHandler {
    fn handle_key_event(&mut self, app: &mut App, key: KeyEvent) -> io::Result<bool>; // Changed from key_code: KeyCode
}