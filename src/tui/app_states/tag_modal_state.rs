use crate::{
    db,
    tui::{
        app::App,
        app_states::{app_state::InputMode, state::KeyEventHandler, app_state::TagModalFocus},
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
    pub focus: TagModalFocus,
}

impl TagModalState {
    pub fn new(all_tags: &[String]) -> Self {
        Self {
            input: String::new(),
            filtered_tags: all_tags.to_vec(),
            selection: ListState::default(),
            focus: TagModalFocus::Input,
        }
    }

    /// Update the filtered tags based on the input.
    pub fn update_filtered_tags(&mut self, source_tags: &[String]) {
        let matcher = SkimMatcherV2::default();
        if self.input.is_empty() {
            self.filtered_tags = source_tags.to_vec();
        } else {
            self.filtered_tags = source_tags
                .iter()
                .filter(|tag| matcher.fuzzy_match(tag, &self.input).is_some())
                .cloned()
                .collect();
        }
        self.selection
            .select(if self.filtered_tags.is_empty() { None } else { Some(0) });
    }
    fn update_tags_from_current_mode(&mut self, app: &mut App) {
        match app.input_mode {
            InputMode::Tagging => {
                self.update_filtered_tags(&app.state.all_tags);
            }
            InputMode::Untagging => {
                // Get the *original* list of tags for the selected package
                let package_tags = if let Some(selected_index) = app.selected_package.selected() {
                    app.state
                        .filtered_packages
                        .get(selected_index)
                        .map(|p| p.tags.clone())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                self.update_filtered_tags(&package_tags);
            }
            _ => {
                // Default case
                self.update_filtered_tags(&app.state.all_tags);
            }
        }
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
            focus: TagModalFocus::Input,
        }
    }
}
impl KeyEventHandler for TagModalState {
    fn handle_key_event(&mut self, app: &mut App, key_code: KeyCode) -> io::Result<bool> {
        match key_code {
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
                self.focus = TagModalFocus::Input; // Reset focus
            }
            KeyCode::Esc => {
                self.input.clear();
                self.update_filtered_tags(&app.state.all_tags);
                app.input_mode = InputMode::Normal;
                self.selection.select(None);
                self.focus = TagModalFocus::Input; // Reset focus
            }
            _ => {
                match self.focus {
                    TagModalFocus::Input => match key_code {
                        KeyCode::Char(c) => {
                            self.input.push(c);
                            self.update_tags_from_current_mode(app);
                        }
                        KeyCode::Backspace => {
                            self.input.pop();
                            self.update_tags_from_current_mode(app);
                        }
                        KeyCode::Tab => {
                            self.focus = TagModalFocus::List;
                        }
                        _ => {}
                    },
                    TagModalFocus::List => match key_code {
                        KeyCode::Char('q') => {
                            self.input.clear();
                            self.update_filtered_tags(&app.state.all_tags);
                            app.input_mode = InputMode::Normal;
                            self.selection.select(None);
                            self.focus = TagModalFocus::Input; // Reset focus
                        }
                        KeyCode::Up | KeyCode::Char('k') => self.select_previous_tag(),
                        KeyCode::Down | KeyCode::Char('j') => self.select_next_tag(),
                        KeyCode::Tab => {
                            self.focus = TagModalFocus::Input;
                        }
                        _ => {}
                    },
                }
            }
        }
        Ok(false)
    }
}
