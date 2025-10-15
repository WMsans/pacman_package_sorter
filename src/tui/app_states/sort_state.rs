use crate::{
    packages::models::{SortKey},
};
use ratatui::widgets::ListState;

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
            Some(i) => if i == 0 { self.options.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.selection.select(Some(i));
    }

    pub fn select_next(&mut self) {
        let i = match self.selection.selected() {
            Some(i) => if i >= self.options.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.selection.select(Some(i));
    }
}