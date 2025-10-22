use crate::{

    config::Action, 
    tui::{
        app::App,
        app_states::{
            app_state::{ActionModalFocus, InputMode, TagModalFocus},
            state::KeyEventHandler,
        },
    },
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::widgets::ListState;
use std::io;

pub struct ActionModalState {
    pub input: String,

    pub all_actions: Vec<Action>, 
    pub filtered_options: Vec<Action>, 
    pub selection: ListState,
    pub focus: ActionModalFocus,
}

impl ActionModalState {
    pub fn new() -> Self {

        Self {
            input: String::new(),

            all_actions: Vec::new(), 
            filtered_options: Vec::new(), 
            selection: ListState::default(),
            focus: ActionModalFocus::Input,
        }
    }

    pub fn load_actions_from_config(&mut self, config: &crate::config::Config) {
        let mut actions = config.actions.clone();

        actions.push(Action::new_local("Add Tag", 'a', false));
        actions.push(Action::new_local("Remove Tag", 'd', false));

        self.all_actions = actions;
        self.update_filtered_options();
    }

    pub fn update_filtered_options(&mut self) {
        let matcher = SkimMatcherV2::default();
        if self.input.is_empty() {
            self.filtered_options = self.all_actions.clone();
        } else {
            self.filtered_options = self.all_actions
                .iter()
                .filter(|action| matcher.fuzzy_match(&action.name, &self.input).is_some())
                .cloned()
                .collect();
        }
        self.selection
            .select(if self.filtered_options.is_empty() { None } else { Some(0) });
    }

    pub fn select_previous(&mut self) {
        if self.filtered_options.is_empty() {
            return;
        }
        let i = match self.selection.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_options.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selection.select(Some(i));
    }

    pub fn select_next(&mut self) {
        if self.filtered_options.is_empty() {
            return;
        }
        let i = match self.selection.selected() {
            Some(i) => {
                if i >= self.filtered_options.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selection.select(Some(i));
    }

    fn on_enter(&mut self, app: &mut App) -> bool {
        if let Some(selected_index) = self.selection.selected() {

            if let Some(action) = self.filtered_options.get(selected_index).cloned() {
                match &action.action_type {

                    crate::config::ActionType::Local => {
                        match action.name.as_str() {
                            "Add Tag" => {
                                app.input_mode = InputMode::Tagging;
                                app.tag_state.update_filtered_tags(&app.state.all_tags);
                                app.tag_state.selection.select(Some(0));
                                app.tag_state.input.clear();
                                app.tag_state.focus = TagModalFocus::Input;
                                return false; 
                            }
                            "Remove Tag" => {
                                let package_tags =
                                    if let Some(pkg_idx) = app.selected_package.selected() {
                                        app.state
                                            .filtered_packages
                                            .get(pkg_idx)
                                            .map(|p| p.tags.clone())
                                            .unwrap_or_default()
                                    } else {
                                        Vec::new()
                                    };

                                if !package_tags.is_empty() {
                                    app.input_mode = InputMode::Untagging;
                                    app.tag_state.update_filtered_tags(&package_tags);
                                    app.tag_state.selection.select(Some(0));
                                    app.tag_state.input.clear();
                                    app.tag_state.focus = TagModalFocus::Input;
                                } else {
                                    app.output.warn(
                                        "Selected package has no tags to remove.".to_string(),
                                    );
                                    app.input_mode = InputMode::Normal;
                                }
                                return false; 
                            }
                            _ => {
                                app.input_mode = InputMode::Normal;
                                return false;
                            }
                        }
                    }

                    crate::config::ActionType::Command { .. } => {

                        return app.execute_config_action(&action);
                    }
                }
            }
        }
        app.input_mode = InputMode::Normal;
        false
    }
}

impl Default for ActionModalState {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyEventHandler for ActionModalState {
    fn handle_key_event(&mut self, app: &mut App, key: KeyEvent) -> io::Result<bool> {
        match self.focus {
            ActionModalFocus::Input => {

                if key.modifiers == KeyModifiers::CONTROL {
                    match key.code {

                        KeyCode::Char('w') | KeyCode::Char('h') => {
                            delete_word_backward(&mut self.input);
                            self.update_filtered_options();
                            return Ok(false);
                        }
                        _ => {}
                    }
                }

                match key.code {
                    KeyCode::Char(c) => {
                        self.input.push(c);
                        self.update_filtered_options();
                    }
                    KeyCode::Backspace => {
                        self.input.pop();
                        self.update_filtered_options();
                    }
                    KeyCode::Tab | KeyCode::Down => {
                        self.focus = ActionModalFocus::List;
                        self.selection.select(Some(0));
                    }
                    KeyCode::Enter => {

                        if self.filtered_options.len() == 1 {
                            self.selection.select(Some(0));

                            if self.on_enter(app) {
                                return Ok(true); 
                            }
                        } else {

                            self.focus = ActionModalFocus::List;
                            self.selection.select(Some(0));
                        }
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                }
            }
            ActionModalFocus::List => match key.code {
                KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
                KeyCode::Down | KeyCode::Char('j') => self.select_next(),
                KeyCode::Tab => {
                    self.focus = ActionModalFocus::Input;
                }
                KeyCode::Enter => {

                    if self.on_enter(app) {
                        return Ok(true); 
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.input_mode = InputMode::Normal;
                }
                _ => {}
            },
        }
        Ok(false)
    }
}

fn delete_word_backward(text: &mut String) {
    let original_len = text.len();
    if original_len == 0 {
        return;
    }

    let trimmed_len = text.trim_end().len();

    let new_len = text[..trimmed_len]
        .rfind(' ')
        .map_or(0, |i| i + 1); 

    text.truncate(new_len);
}