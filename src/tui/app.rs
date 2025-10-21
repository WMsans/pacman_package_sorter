use crate::{backend, config, db}; 
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::prelude::*;
use ratatui::widgets::ListState;
use ratatui::Terminal;
use std::io::Stdout;
use tokio::sync::mpsc;

use crate::packages::models::ShowMode;
use crate::tui::app_states::{
    app_state::{AppState, InputMode, LoadedData},
    filter_modal_state::FilterModalState,
    normal_state::NormalState,
    action_modal_state::ActionModalState,
    search_state::SearchState,
    show_mode_state::ShowModeState,
    sort_state::SortState,
    tag_modal_state::TagModalState,
};
use crate::tui::event::handle_events;
use crate::tui::ui;

// --- Main Application Struct ---

pub struct App {
    pub state: AppState,
    pub selected_package: ListState,
    pub input_mode: InputMode,
    pub output: Vec<String>,
    pub action_state: ActionModalState,
    pub command_to_run: Option<Vec<String>>,
    pub config: config::Config, // <-- ADD THIS

    // Search
    pub search_input: String,
    pub search_cursor_position: usize,

    // UI states
    pub sort_state: SortState,
    pub filter_state: FilterModalState,
    pub tag_state: TagModalState,
    pub normal_state: NormalState,
    pub search_state: SearchState,
    pub show_mode_state: ShowModeState,

    pub data_receiver: mpsc::Receiver<LoadedData>,
    pub is_loading: bool,
}

impl App {
    pub fn new(rx: mpsc::Receiver<LoadedData>) -> Self {
        let state = AppState::new();
        let sort_state = SortState::new();
        let filter_state = FilterModalState::new(&state.all_tags, &state.all_repos);
        let tag_state = TagModalState::new(&state.all_tags);
        let show_mode_state = ShowModeState::new();
        let action_state = ActionModalState::new();
        let config = config::load_config(); // <-- ADD THIS

        App {
            state,
            selected_package: ListState::default(),
            input_mode: InputMode::Normal,
            output: Vec::new(),
            command_to_run: None,
            config, // <-- ADD THIS
            search_input: String::new(),
            search_cursor_position: 0,
            sort_state,
            filter_state,
            tag_state,
            normal_state: NormalState,
            search_state: SearchState,
            show_mode_state,
            action_state,
            data_receiver: rx,
            is_loading: true,
        }
    }

    /// ADD THIS ENTIRE HELPER FUNCTION
    ///
    /// Executes a command action from the config, performing all
    /// necessary checks.
    /// Returns `true` if the TUI should quit to run the command.
    pub fn execute_config_action(&mut self, action: &crate::config::Action) -> bool {
        let (command_template, requires_package, show_mode_whitelist, show_mode_blacklist) =
            match &action.action_type {
                crate::config::ActionType::Command {
                    command,
                    requires_package,
                    show_mode_whitelist,
                    show_mode_blacklist,
                } => (
                    command,
                    requires_package,
                    show_mode_whitelist,
                    show_mode_blacklist,
                ),
                _ => return false, // Not a command action
            };

        // Check whitelist
        if !show_mode_whitelist.is_empty() {
            let current_mode_str = self.show_mode_state.active_show_mode.to_string();
            if !show_mode_whitelist.contains(&current_mode_str) {
                self.output.push(format!(
                    "Action '{}' can only be run in modes: {}",
                    action.name,
                    show_mode_whitelist.join(", ")
                ));
                self.input_mode = InputMode::Normal; // Ensure modal closes
                return false;
            }
        }

        // Check blacklist
        if !show_mode_blacklist.is_empty() {
            let current_mode_str = self.show_mode_state.active_show_mode.to_string();
            if show_mode_blacklist.contains(&current_mode_str) {
                self.output.push(format!(
                    "Action '{}' cannot be run in mode: {}",
                    action.name, current_mode_str
                ));
                self.input_mode = InputMode::Normal;
                return false;
            }
        }

        let mut package_name: Option<String> = None;
        if *requires_package {
            if let Some(selected_index) = self.selected_package.selected() {
                if let Some(package) = self.state.filtered_packages.get(selected_index) {
                    package_name = Some(package.name.clone());
                }
            }

            if package_name.is_none() {
                self.output.push(format!(
                    "Action '{}' requires a selected package.",
                    action.name
                ));
                self.input_mode = InputMode::Normal;
                return false;
            }
        }

        let final_command =
            crate::config::template_command(command_template, package_name.as_deref())
                .unwrap_or_default();

        self.command_to_run = Some(final_command);
        true // Quit TUI to run command
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> std::io::Result<()> {
        
        loop {
            // On each loop, check if we are still loading
            if self.is_loading {
                // Try to receive data from the channel non-blockingly
                if let Ok(loaded_data) = self.data_receiver.try_recv() {
                    // If we get data, update the app state
                    self.state.packages = loaded_data.packages;
                    self.state.available_packages = loaded_data.available_packages;
                    self.state.all_repos = loaded_data.all_repos;
                    self.state.orphan_package_names = loaded_data.orphan_package_names;
                    
                    // Reload tags from DB
                    self.reload_tags(); 
                    
                    // Update filter and tag states with the new data
                    self.filter_state = FilterModalState::new(&self.state.all_tags, &self.state.all_repos);
                    self.tag_state = TagModalState::new(&self.state.all_tags);

                    // We are no longer loading
                    self.is_loading = false;
                    self.apply_filters(); // Apply initial filters/sort to show the list
                }
            }

            terminal.draw(|f| ui::ui(f, self))?;
            
            // handle_events is now non-blocking, so this loop will spin
            if handle_events(self)? {
                break; // Exit the loop (and the function)
            }
        }
        Ok(())
    }

    pub fn apply_filters(&mut self) {
        // Decide which base list of packages to use
        let source_list = if self.show_mode_state.active_show_mode == ShowMode::AllAvailable {
            &self.state.available_packages
        } else {
            &self.state.packages
        };

        self.state.filtered_packages = backend::filter_packages(
            source_list, // Pass the chosen list
            &self.filter_state.tag_filters,
            &self.filter_state.repo_filters,
            self.show_mode_state.active_show_mode,
            &self.state.orphan_package_names,
        );

        if !self.search_input.is_empty() {
            let matcher = SkimMatcherV2::default();
            self.state.filtered_packages = self.state
                .filtered_packages
                .iter()
                .filter(|pkg| matcher.fuzzy_match(&pkg.name, &self.search_input).is_some())
                .cloned()
                .collect();
        }
        self.sort_packages();
        if !self.state.filtered_packages.is_empty() {
            self.selected_package.select(Some(0));
        } else {
            self.selected_package.select(None);
        }
    }

    pub fn sort_packages(&mut self) {
        backend::sort_packages(
            &mut self.state.filtered_packages,
            self.sort_state.active_sort_key,
        );
    }

    pub fn select_previous_package(&mut self) {
        let i = match self.selected_package.selected() {
            Some(i) => {
                if i == 0 {
                    self.state.filtered_packages.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected_package.select(Some(i));
    }

    pub fn select_next_package(&mut self) {
        let i = match self.selected_package.selected() {
            Some(i) => {
                if i >= self.state.filtered_packages.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected_package.select(Some(i));
    }

    pub fn reload_tags(&mut self) {
        self.state.all_tags = db::get_all_tags().unwrap_or_default();
        self.tag_state.update_filtered_tags(&self.state.all_tags);
    }
}