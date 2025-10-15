use crate::{
    db,
    tui::{
        app::App,
        app_states::{app_state::InputMode, state::KeyEventHandler},
    },
};
use crossterm::event::KeyCode;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::widgets::ListState;
use std::io;

/// Manages the state for the tagging/untagging functionality
pub struct TagModalState {
    pub input: String,
    pub filtered_tags: Vec<String>,
    pub selection: ListState,
}

impl TagModalState {
    pub fn new(all_tags: &[String]) -> Self {
        Self {
            input: String::new(),
            filtered_tags: all_tags.to_vec(),
            selection: ListState::default(),
        }
    }

    /// Update the filtered tags based on the input.
    pub fn update_filtered_tags(&mut self, all_tags: &[String]) {
        let matcher = SkimMatcherV2::default();
        if self.input.is_empty() {
            self.filtered_tags = all_tags.to_vec();
        } else {
            self.filtered_tags = all_tags
                .iter()
                .filter(|tag| matcher.fuzzy_match(tag, &self.input).is_some())
                .cloned()
                .collect();
        }
        self.selection
            .select(if self.filtered_tags.is_empty() { None } else { Some(0) });
    }

    pub fn select_previous_tag(&mut self) {
        if self.filtered_tags.is_empty() {
            return;
        }
        let i = match self.selection.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_tags.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selection.select(Some(i));
        if let Some(tag) = self.filtered_tags.get(i) {
            self.input = tag.clone();
        }
    }

    pub fn select_next_tag(&mut self) {
        if self.filtered_tags.is_empty() {
            return;
        }
        let i = match self.selection.selected() {
            Some(i) => {
                if i >= self.filtered_tags.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selection.select(Some(i));
        if let Some(tag) = self.filtered_tags.get(i) {
            self.input = tag.clone();
        }
    }
}

impl Default for TagModalState {
    fn default() -> Self {
        Self {
            input: String::new(),
            filtered_tags: Vec::new(),
            selection: ListState::default(),
        }
    }
}

impl KeyEventHandler for TagModalState {
    fn handle_key_event(&mut self, app: &mut App, key_code: KeyCode) -> io::Result<bool> {
        match key_code {
            KeyCode::Up | KeyCode::Char('k') => self.select_previous_tag(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next_tag(),
            KeyCode::Enter => {
                if let Some(selected_index) = app.selected_package.selected() {
                    if let Some(selected_pkg_name) =
                        app.state.filtered_packages.get(selected_index).map(|p| p.name.clone())
                    {
                        let tag_to_apply = self.input.trim().to_string();
                        if !tag_to_apply.is_empty() {
                            let result = if matches!(app.input_mode, InputMode::Tagging) {
                                db::add_tag(&selected_pkg_name, &tag_to_apply)
                            } else {
                                db::remove_tag(&selected_pkg_name, &tag_to_apply)
                            };
                            match result {
                                Ok(msg) => {
                                    app.output.push(msg);
                                    // Find the package in the main list and update its tags
                                    if let Some(pkg_to_update) = app
                                        .state
                                        .packages
                                        .iter_mut()
                                        .find(|p| p.name == selected_pkg_name)
                                    {
                                        if matches!(app.input_mode, InputMode::Tagging) {
                                            if !pkg_to_update.tags.contains(&tag_to_apply) {
                                                pkg_to_update.tags.push(tag_to_apply);
                                                pkg_to_update.tags.sort();
                                            }
                                        } else {
                                            pkg_to_update.tags.retain(|t| t != &tag_to_apply);
                                        }
                                    }
                                    app.reload_tags();
                                    app.apply_filters(); // Re-apply filters to update the view
                                }
                                Err(e) => {
                                    app.output.push(format!("Error: {}", e));
                                }
                            }
                        }
                    }
                }
                self.input.clear();
                self.update_filtered_tags(&app.state.all_tags);
                app.input_mode = InputMode::Normal;
                self.selection.select(None);
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                self.update_filtered_tags(&app.state.all_tags);
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.update_filtered_tags(&app.state.all_tags);
            }
            KeyCode::Esc => {
                self.input.clear();
                self.update_filtered_tags(&app.state.all_tags);
                app.input_mode = InputMode::Normal;
                self.selection.select(None);
            }
            _ => {}
        }
        Ok(false)
    }
}