use crate::{
    backend,
    tui::app_states::app_state::FilterFocus,
};
use backend::FilterState;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::widgets::ListState;
use std::collections::HashMap;


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
    pub fn new(all_tags: &[String], all_repos: &[String]) -> Self {
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