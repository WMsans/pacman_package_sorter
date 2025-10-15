use crate::{backend, db};
use ratatui::prelude::*;
use ratatui::widgets::ListState;
use ratatui::Terminal;
use std::io::Stdout;

use crate::tui::app_states::{
    app_state::{AppState, InputMode},
    filter_modal_state::FilterModalState,
    normal_state::NormalState,
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
    pub show_explicit: bool,
    pub show_dependency: bool,

    // UI states
    pub sort_state: SortState,
    pub filter_state: FilterModalState,
    pub tag_state: TagModalState,
    pub normal_state: NormalState,
}

impl App {
    pub fn new() -> Self {
        let state = AppState::new();
        let sort_state = SortState::new();
        let filter_state = FilterModalState::new(&state.all_tags, &state.all_repos);
        let tag_state = TagModalState::new(&state.all_tags);

        App {
            state,
            selected_package: ListState::default(),
            input_mode: InputMode::Normal,
            output: Vec::new(),
            show_explicit: false,
            show_dependency: false,
            sort_state,
            filter_state,
            tag_state,
            normal_state: NormalState,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> std::io::Result<()> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            self.state.load_packages().await;
        });

        // Initialize filter and tag states with loaded data
        self.filter_state = FilterModalState::new(&self.state.all_tags, &self.state.all_repos);
        self.tag_state = TagModalState::new(&self.state.all_tags);

        self.apply_filters();

        if !self.state.filtered_packages.is_empty() {
            self.selected_package.select(Some(0));
        }

        loop {
            terminal.draw(|f| ui::ui(f, self))?;
            if handle_events(self)? {
                break;
            }
        }
        Ok(())
    }

    pub fn apply_filters(&mut self) {
        self.state.filtered_packages = backend::filter_packages(
            &self.state.packages,
            &self.filter_state.tag_filters,
            &self.filter_state.repo_filters,
            self.show_explicit,
            self.show_dependency,
        );
        self.sort_packages();
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