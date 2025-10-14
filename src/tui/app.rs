use crate::{backend, db, models::Package, models::SortKey};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::prelude::*;
use ratatui::widgets::ListState;
use ratatui::Terminal;
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

pub struct App {
    pub packages: Vec<Package>,
    pub filtered_packages: Vec<Package>,
    pub selected_package: ListState,
    pub sort_key: SortKey,
    pub include_tags: Vec<String>,
    pub exclude_tags: Vec<String>,
    pub include_repos: Vec<String>,
    pub exclude_repos: Vec<String>,
    pub input: String,
    pub input_mode: InputMode,
    pub show_explicit: bool,
    pub show_dependency: bool,
    pub output: Vec<String>,
    pub all_tags: Vec<String>,
    pub filtered_tags: Vec<String>,
    pub tag_selection: ListState,
    pub sort_options: Vec<SortKey>,
    pub sort_selection: ListState,
    pub filter_input: String,
    pub filter_cursor_position: usize,
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
            include_tags: Vec::new(),
            exclude_tags: Vec::new(),
            include_repos: Vec::new(),
            exclude_repos: Vec::new(),
            input: String::new(),
            input_mode: InputMode::Normal,
            show_explicit: false,
            show_dependency: false,
            output: Vec::new(),
            filtered_tags: all_tags.clone(),
            all_tags,
            tag_selection: ListState::default(),
            sort_options,
            sort_selection: ListState::default(),
            filter_input: String::new(),
            filter_cursor_position: 0,
        }
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> std::io::Result<()> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            self.packages = backend::get_all_packages().await.unwrap_or_default();
        });
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
            &self.include_tags,
            &self.exclude_tags,
            &self.include_repos,
            &self.exclude_repos,
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
        self.tag_selection.select(if self.filtered_tags.is_empty() { None } else { Some(0) });
    }
}