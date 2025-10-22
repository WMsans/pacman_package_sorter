use crate::tui::app::App;
use crate::tui::app_states::app_state::{ActionModalFocus, InputMode};
use crate::tui::app_states::state::KeyEventHandler;
use crate::tui::app_states::app_state::TagModalFocus;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers}; 
use std::io;

#[derive(Clone, Copy)]
pub struct NormalState;

impl KeyEventHandler for NormalState {
    fn handle_key_event(&mut self, app: &mut App, key: KeyEvent) -> io::Result<bool> {
        let (key_char, shift) = match key.code {
            KeyCode::Char(c) => (c, key.modifiers == KeyModifiers::SHIFT),
            _ => ('\0', false), 
        };

        if key_char != '\0' {
            for action in app.config.actions.clone() {
                if action.key.key == key_char && action.key.shift == shift {
                    if let crate::config::ActionType::Command { .. } = action.action_type {
                        return Ok(app.execute_config_action(&action));
                    }
                }
            }
        }

        if key.modifiers == KeyModifiers::SHIFT {
            match key.code {

                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('k') => app.select_previous_package(), 
            KeyCode::Char('j') => app.select_next_package(),   
            KeyCode::Up => app.output.scroll_up(1),               
            KeyCode::Down => app.output.scroll_down(1),             
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
                    app.output
                        .warn("Selected package has no tags to remove.".to_string());
                }
            }

            KeyCode::Char('?') => {
                app.input_mode = InputMode::Action;

                app.action_state.load_actions_from_config(&app.config);

                app.action_state.input.clear();
                app.action_state.update_filtered_options(); 
                app.action_state.selection.select(Some(0));
                app.action_state.focus = ActionModalFocus::Input;
            }
            _ => {}
        }
        Ok(false)
    }
}