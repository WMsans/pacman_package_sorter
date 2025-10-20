use crate::{
    packages::models::SortKey,
    tui::{
        app::App,
        app_states::{app_state::InputMode, state::KeyEventHandler},
    },
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use std::io;

/// Manages the state for the sorting functionality
pub struct SortState {
    pub options: Vec<SortKey>,
    pub selection: ListState,
    pub active_sort_key: SortKey,
}

impl SortState {
    pub fn new() -> Self {
        Self {
            options: vec![
                SortKey::Name,
                SortKey::Size,
                SortKey::InstallDate,
                SortKey::UpdateDate,
                SortKey::Popularity,
            ],
            selection: ListState::default(),
            active_sort_key: SortKey::Name,
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
}

impl Default for SortState {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyEventHandler for SortState {
    fn handle_key_event(&mut self, app: &mut App, key: KeyEvent) -> io::Result<bool> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Enter => {
                if let Some(selected) = self.selection.selected() {
                    if let Some(sort_key) = self.options.get(selected) {
                        self.active_sort_key = *sort_key;
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