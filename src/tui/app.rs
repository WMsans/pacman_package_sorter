use crate::{
    backend, db,
    packages::models::{Package, SortKey},
};
use backend::FilterState;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::prelude::*;
use ratatui::widgets::ListState;
use ratatui::Terminal;
use std::collections::HashMap;
use std::io::Stdout;

use crate::tui::event::handle_events;
use crate::tui::ui;

// --- Enums for application state ---

pub enum InputMode {
    Normal,
    Tagging,
    Untagging,
    Sorting,
    Filtering,
}

pub enum FilterFocus {
    Search,
    Tags,
    Repos,
}

// --- State Management Structs ---

/// Holds the core data of the application
pub struct AppState {
    pub packages: Vec<Package>,
    pub filtered_packages: Vec<Package>,
    pub all_tags: Vec<String>,
    pub all_repos: Vec<String>,
}

impl AppState {
    fn new() -> Self {
        Self {
            packages: Vec::new(),
            filtered_packages: Vec::new(),
            all_tags: db::get_all_tags().unwrap_or_default(),
            all_repos: Vec::new(),
        }
    }

    /// Reloads packages and repos from the backend.
    async fn load_packages(&mut self) {
        self.packages = crate::packages::pacman::get_all_packages()
            .await
            .unwrap_or_default();
        self.all_repos = backend::get_all_repos(&self.packages);
    }
}

/// Manages the state for the sorting functionality
pub struct SortState {
    pub options: Vec<SortKey>,
    pub selection: ListState,
    pub active_sort_key: SortKey,
}

impl SortState {
    fn new() -> Self {
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

/// Manages the state for the filtering functionality
pub struct FilterModalState {
    pub input: String,
    pub cursor_position: usize,
    pub focus: FilterFocus,
    pub tag_filters: HashMap<String, FilterState>,
    pub repo_filters: HashMap<String, FilterState>,
    pub filtered_tags: Vec<String>,
    pub filtered_repos: Vec<String>,
    pub tag_selection: ListState,
    pub repo_selection: ListState,
}

impl FilterModalState {
    fn new(all_tags: &[String], all_repos: &[String]) -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
            focus: FilterFocus::Search,
            tag_filters: HashMap::new(),
            repo_filters: HashMap::new(),
            filtered_tags: all_tags.to_vec(),
            filtered_repos: all_repos.to_vec(),
            tag_selection: ListState::default(),
            repo_selection: ListState::default(),
        }
    }

    /// Update the filtered tags and repos based on the input.
    pub fn update_filtered_options(&mut self, all_tags: &[String], all_repos: &[String]) {
        let matcher = SkimMatcherV2::default();
        if self.input.is_empty() {
            self.filtered_tags = all_tags.to_vec();
            self.filtered_repos = all_repos.to_vec();
        } else {
            self.filtered_tags = all_tags
                .iter()
                .filter(|tag| matcher.fuzzy_match(tag, &self.input).is_some())
                .cloned()
                .collect();
            self.filtered_repos = all_repos
                .iter()
                .filter(|repo| matcher.fuzzy_match(repo, &self.input).is_some())
                .cloned()
                .collect();
        }
        self.tag_selection.select(if self.filtered_tags.is_empty() { None } else { Some(0) });
        self.repo_selection.select(if self.filtered_repos.is_empty() { None } else { Some(0) });
    }
    
    /// Cycle through filter states (Include, Exclude, Ignore).
    pub fn cycle_filter_state(&mut self, forward: bool) {
        let (selected_index, items, filters) = match self.focus {
            FilterFocus::Tags => (
                self.tag_selection.selected(),
                &self.filtered_tags,
                &mut self.tag_filters,
            ),
            FilterFocus::Repos => (
                self.repo_selection.selected(),
                &self.filtered_repos,
                &mut self.repo_filters,
            ),
            FilterFocus::Search => return,
        };

        if let Some(selected) = selected_index {
            if let Some(key) = items.get(selected) {
                let current_state = filters.get(key).cloned().unwrap_or_default();
                let next_state = if forward {
                    match current_state {
                        FilterState::Ignore => FilterState::Include,
                        FilterState::Include => FilterState::Exclude,
                        FilterState::Exclude => FilterState::Ignore,
                    }
                } else {
                    match current_state {
                        FilterState::Ignore => FilterState::Exclude,
                        FilterState::Exclude => FilterState::Include,
                        FilterState::Include => FilterState::Ignore,
                    }
                };
                filters.insert(key.clone(), next_state);
            }
        }
    }
}

/// Manages the state for the tagging/untagging functionality
pub struct TagModalState {
    pub input: String,
    pub filtered_tags: Vec<String>,
    pub selection: ListState,
}

impl TagModalState {
    fn new(all_tags: &[String]) -> Self {
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
        self.selection.select(if self.filtered_tags.is_empty() { None } else { Some(0) });
    }
    
    pub fn select_previous_tag(&mut self) {
        if self.filtered_tags.is_empty() { return; }
        let i = match self.selection.selected() {
            Some(i) => if i == 0 { self.filtered_tags.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.selection.select(Some(i));
        if let Some(tag) = self.filtered_tags.get(i) {
            self.input = tag.clone();
        }
    }

    pub fn select_next_tag(&mut self) {
        if self.filtered_tags.is_empty() { return; }
        let i = match self.selection.selected() {
            Some(i) => if i >= self.filtered_tags.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.selection.select(Some(i));
        if let Some(tag) = self.filtered_tags.get(i) {
            self.input = tag.clone();
        }
    }
}


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
        backend::sort_packages(&mut self.state.filtered_packages, self.sort_state.active_sort_key);
    }

    pub fn select_previous_package(&mut self) {
        let i = match self.selected_package.selected() {
            Some(i) => if i == 0 { self.state.filtered_packages.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.selected_package.select(Some(i));
    }

    pub fn select_next_package(&mut self) {
        let i = match self.selected_package.selected() {
            Some(i) => if i >= self.state.filtered_packages.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.selected_package.select(Some(i));
    }
    
    pub fn reload_tags(&mut self) {
        self.state.all_tags = db::get_all_tags().unwrap_or_default();
        self.tag_state.update_filtered_tags(&self.state.all_tags);
    }
}