use crate::{
    packages::models::ShowMode,
    tui::{
        app::App,
        app_states::{app_state::InputMode, state::KeyEventHandler},
    },
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use std::io;

/// Manages the state for the show mode functionality
pub struct ShowModeState {
    pub options: Vec<ShowMode>,
    pub selection: ListState,
    pub active_show_mode: ShowMode,
}

impl ShowModeState {
    pub fn new() -> Self {
        Self {
            options: vec![
                ShowMode::AllInstalled,
                ShowMode::ExplicitlyInstalled,
                ShowMode::Dependencies,
                ShowMode::Orphans,
                ShowMode::AllAvailable, 
            ],
            selection: ListState::default(),
            active_show_mode: ShowMode::AllInstalled,
        }
    }

    pub fn select_previous(&mut self) {
        let i = match self.selection.selected() {
            Some(i) => {
                if i == 0 {
                    self.options.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selection.select(Some(i));
    }

    pub fn select_next(&mut self) {
        let i = match self.selection.selected() {
            Some(i) => {
                if i >= self.options.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selection.select(Some(i));
    }

    /// Sets the selection to the currently active show mode
    pub fn select_active(&mut self) {
        if let Some(index) = self.options.iter().position(|&s| s == self.active_show_mode) {
            self.selection.select(Some(index));
        } else {
            self.selection.select(Some(0));
        }
    }
}

impl Default for ShowModeState {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyEventHandler for ShowModeState {
    fn handle_key_event(&mut self, app: &mut App, key: KeyEvent) -> io::Result<bool> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Enter => {
                if let Some(selected) = self.selection.selected() {
                    if let Some(show_mode) = self.options.get(selected) {
                        self.active_show_mode = *show_mode;
                    }
                }
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(false)
    }
}