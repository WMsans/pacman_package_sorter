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

pub struct App {
    pub packages: Vec<Package>,
    pub filtered_packages: Vec<Package>,
    pub selected_package: ListState,
    pub sort_key: SortKey,
    pub tag_filters: HashMap<String, FilterState>,
    pub repo_filters: HashMap<String, FilterState>,
    pub input: String,
    pub input_mode: InputMode,
    pub show_explicit: bool,
    pub show_dependency: bool,
    pub output: Vec<String>,
    pub all_tags: Vec<String>,
    pub all_repos: Vec<String>,
    pub filtered_tags: Vec<String>,
    pub filtered_repos: Vec<String>,
    pub tag_selection: ListState,
    pub sort_options: Vec<SortKey>,
    pub sort_selection: ListState,
    pub filter_input: String,
    pub filter_cursor_position: usize,
    pub tag_filter_selection: ListState,
    pub repo_filter_selection: ListState,
    pub filter_focus: FilterFocus,
}

impl App {
    pub fn new() -> Self {
        let all_tags = db::get_all_tags().unwrap_or_default();
        let sort_options = vec![
            SortKey::Name,
            SortKey::Size,
            SortKey::InstallDate,
            SortKey::UpdateDate,
            SortKey::Popularity,
        ];
        App {
            packages: Vec::new(),
            filtered_packages: Vec::new(),
            selected_package: ListState::default(),
            sort_key: SortKey::Name,
            tag_filters: HashMap::new(),
            repo_filters: HashMap::new(),
            input: String::new(),
            input_mode: InputMode::Normal,
            show_explicit: false,
            show_dependency: false,
            output: Vec::new(),
            filtered_tags: all_tags.clone(),
            all_tags,
            all_repos: Vec::new(),
            filtered_repos: Vec::new(),
            tag_selection: ListState::default(),
            sort_options,
            sort_selection: ListState::default(),
            filter_input: String::new(),
            filter_cursor_position: 0,
            tag_filter_selection: ListState::default(),
            repo_filter_selection: ListState::default(),
            filter_focus: FilterFocus::Search,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> std::io::Result<()> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            self.packages = crate::packages::pacman::get_all_packages()
                .await
                .unwrap_or_default();
        });
        self.all_repos = backend::get_all_repos(&self.packages);
        self.apply_filters();

        if !self.filtered_packages.is_empty() {
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
        self.filtered_packages = backend::filter_packages(
            &self.packages,
            &self.tag_filters,
            &self.repo_filters,
            self.show_explicit,
            self.show_dependency,
        );
        self.sort_packages();
    }

    pub fn sort_packages(&mut self) {
        backend::sort_packages(&mut self.filtered_packages, self.sort_key);
    }

    pub fn select_previous(&mut self) {
        let i = match self.selected_package.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_packages.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected_package.select(Some(i));
    }

    pub fn select_next(&mut self) {
        let i = match self.selected_package.selected() {
            Some(i) => {
                if i >= self.filtered_packages.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected_package.select(Some(i));
    }

    pub fn select_previous_tag(&mut self) {
        if self.filtered_tags.is_empty() {
            return;
        }
        let i = match self.tag_selection.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_tags.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.tag_selection.select(Some(i));
        if let Some(tag) = self.filtered_tags.get(i) {
            self.input = tag.clone();
        }
    }

    pub fn select_next_tag(&mut self) {
        if self.filtered_tags.is_empty() {
            return;
        }
        let i = match self.tag_selection.selected() {
            Some(i) => {
                if i >= self.filtered_tags.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.tag_selection.select(Some(i));
        if let Some(tag) = self.filtered_tags.get(i) {
            self.input = tag.clone();
        }
    }

    pub fn select_previous_sort(&mut self) {
        let i = match self.sort_selection.selected() {
            Some(i) => {
                if i == 0 {
                    self.sort_options.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.sort_selection.select(Some(i));
    }

    pub fn select_next_sort(&mut self) {
        let i = match self.sort_selection.selected() {
            Some(i) => {
                if i >= self.sort_options.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.sort_selection.select(Some(i));
    }

    pub fn reload_tags(&mut self) {
        self.all_tags = db::get_all_tags().unwrap_or_default();
        self.update_filtered_tags();
    }

    pub fn update_filtered_tags(&mut self) {
        let matcher = SkimMatcherV2::default();
        if self.input.is_empty() {
            self.filtered_tags = self.all_tags.clone();
        } else {
            self.filtered_tags = self
                .all_tags
                .iter()
                .filter(|tag| matcher.fuzzy_match(tag, &self.input).is_some())
                .cloned()
                .collect();
        }
        self.tag_selection.select(if self.filtered_tags.is_empty() {
            None
        } else {
            Some(0)
        });
    }

    pub fn update_filtered_filter_options(&mut self) {
        let matcher = SkimMatcherV2::default();
        if self.filter_input.is_empty() {
            self.filtered_tags = self.all_tags.clone();
            self.filtered_repos = self.all_repos.clone();
        } else {
            self.filtered_tags = self
                .all_tags
                .iter()
                .filter(|tag| matcher.fuzzy_match(tag, &self.filter_input).is_some())
                .cloned()
                .collect();
            self.filtered_repos = self
                .all_repos
                .iter()
                .filter(|repo| matcher.fuzzy_match(repo, &self.filter_input).is_some())
                .cloned()
                .collect();
        }
        self.tag_filter_selection
            .select(if self.filtered_tags.is_empty() {
                None
            } else {
                Some(0)
            });
        self.repo_filter_selection
            .select(if self.filtered_repos.is_empty() {
                None
            } else {
                Some(0)
            });
    }

    pub fn cycle_filter_state(&mut self, forward: bool) {
        let (selected_index, items, filters) = match self.filter_focus {
            FilterFocus::Tags => (
                self.tag_filter_selection.selected(),
                &self.filtered_tags,
                &mut self.tag_filters,
            ),
            FilterFocus::Repos => (
                self.repo_filter_selection.selected(),
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