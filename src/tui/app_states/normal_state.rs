use crate::tui::app::App;
use crate::tui::app_states::app_state::{InputMode, ActionModalFocus};
use crate::tui::app_states::state::KeyEventHandler;
use crate::tui::app_states::app_state::TagModalFocus;
use crossterm::event::{KeyCode, KeyEvent};
use std::io;

#[derive(Clone, Copy)]
pub struct NormalState;

impl KeyEventHandler for NormalState {
    fn handle_key_event(&mut self, app: &mut App, key: KeyEvent) -> io::Result<bool> {
        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Up | KeyCode::Char('k') => app.select_previous_package(),
            KeyCode::Down | KeyCode::Char('j') => app.select_next_package(),
            KeyCode::Char('s') => {
                app.input_mode = InputMode::Sorting;
                app.sort_state.selection.select(Some(0));
            }
            KeyCode::Char('v') => {
                app.input_mode = InputMode::Showing;
                app.show_mode_state.select_active();
            }
            KeyCode::Char('f') => {
                app.input_mode = InputMode::Filtering;
                app.filter_state
                    .update_filtered_options(&app.state.all_tags, &app.state.all_repos);
            }
            KeyCode::Char('/') => { 
                app.input_mode = InputMode::Searching;
                app.search_cursor_position = app.search_input.len();
            }
            KeyCode::Char('a') => {
                app.input_mode = InputMode::Tagging;
                app.tag_state.update_filtered_tags(&app.state.all_tags);
                app.tag_state.selection.select(Some(0));
                app.tag_state.input.clear(); 
                app.tag_state.focus = TagModalFocus::Input;
            }
            KeyCode::Char('d') => {
                let package_tags = if let Some(selected_index) = app.selected_package.selected() {
                    app.state
                        .filtered_packages
                        .get(selected_index)
                        .map(|p| p.tags.clone()) // Get the tags
                        .unwrap_or_default() // Or an empty vec
                } else {
                    Vec::new() 
                };

                // Only enter untagging mode if there are tags to remove
                if !package_tags.is_empty() {
                    app.input_mode = InputMode::Untagging;
                    app.tag_state.update_filtered_tags(&package_tags); // Use package's tags
                    app.tag_state.selection.select(Some(0));
                    app.tag_state.input.clear(); // --- MODIFIED ---
                    app.tag_state.focus = TagModalFocus::Input;
                } else {
                    app.output
                        .push("Selected package has no tags to remove.".to_string());
                }
            }
            KeyCode::Char('?') => {
                app.input_mode = InputMode::Action;
                // Reset its state every time it's opened
                app.action_state.input.clear();
                app.action_state.update_filtered_options(); // Use the default options
                app.action_state.selection.select(Some(0));
                app.action_state.focus = ActionModalFocus::Input;
            }
            _ => {}
        }
        Ok(false)
    }
}