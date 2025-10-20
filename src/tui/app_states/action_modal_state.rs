use crate::{
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

const ACTIONS: [&str; 2] = ["Add Tag", "Remove Tag"];

pub struct ActionModalState {
    pub input: String,
    pub options: Vec<String>,
    pub filtered_options: Vec<String>,
    pub selection: ListState,
    pub focus: ActionModalFocus,
}

impl ActionModalState {
    pub fn new() -> Self {
        let options: Vec<String> = ACTIONS.iter().map(|s| s.to_string()).collect();
        Self {
            input: String::new(),
            filtered_options: options.clone(),
            options,
            selection: ListState::default(),
            focus: ActionModalFocus::Input,
        }
    }

    /// Update the filtered options based on the input.
    pub fn update_filtered_options(&mut self) {
        let matcher = SkimMatcherV2::default();
        if self.input.is_empty() {
            self.filtered_options = self.options.clone();
        } else {
            self.filtered_options = self.options
                .iter()
                .filter(|opt| matcher.fuzzy_match(opt, &self.input).is_some())
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

    fn on_enter(&mut self, app: &mut App) {
        if let Some(selected_index) = self.selection.selected() {
             if let Some(action) = self.filtered_options.get(selected_index) {
                match action.as_str() {
                    "Add Tag" => {
                        app.input_mode = InputMode::Tagging;
                        app.tag_state.update_filtered_tags(&app.state.all_tags);
                        app.tag_state.selection.select(Some(0));
                        app.tag_state.input.clear();
                        app.tag_state.focus = TagModalFocus::Input;
                    }
                    "Remove Tag" => {
                        // Logic copied from normal_state.rs 'd' key
                        let package_tags = if let Some(pkg_idx) = app.selected_package.selected() {
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
                            app.output.push("Selected package has no tags to remove.".to_string());
                            app.input_mode = InputMode::Normal; // Stay in normal mode
                        }
                    }
                    _ => {
                        // Should not happen
                        app.input_mode = InputMode::Normal;
                    }
                }
             }
        } else {
            app.input_mode = InputMode::Normal;
        }
    }
}

// ADD THIS BLOCK
impl Default for ActionModalState {
    fn default() -> Self {
        Self::new()
    }
}


impl KeyEventHandler for ActionModalState {
    fn handle_key_event(&mut self, app: &mut App, key: KeyEvent) -> io::Result<bool> {
        match self.focus {
            ActionModalFocus::Input => {
                // Handle modifier keys first
                if key.modifiers == KeyModifiers::CONTROL {
                    match key.code {
                        // Ctrl + W or Ctrl + Backspace
                        KeyCode::Char('w') | KeyCode::Char('h') => {
                            delete_word_backward(&mut self.input);
                            self.update_filtered_options();
                            return Ok(false);
                        }
                        _ => {}
                    }
                }

                // Handle non-modifier keys
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
                       // If only one option and it's selected, just select it.
                       if self.filtered_options.len() == 1 {
                            self.selection.select(Some(0));
                            self.on_enter(app);
                       } else {
                            // Otherwise, just move to the list
                            self.focus = ActionModalFocus::List;
                            self.selection.select(Some(0));
                       }
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                }
            },
            ActionModalFocus::List => match key.code {
                KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
                KeyCode::Down | KeyCode::Char('j') => self.select_next(),
                KeyCode::Tab => {
                    self.focus = ActionModalFocus::Input;
                }
                KeyCode::Enter => {
                    self.on_enter(app);
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

// Added this function from tag_modal_state.rs
fn delete_word_backward(text: &mut String) {
    let original_len = text.len();
    if original_len == 0 {
        return;
    }
    // Find the length after trimming trailing spaces
    let trimmed_len = text.trim_end().len();
    // Find the last space in the trimmed string
    let new_len = text[..trimmed_len]
        .rfind(' ')
        .map_or(0, |i| i + 1); // +1 to be after the space, or 0
    
    text.truncate(new_len);
}