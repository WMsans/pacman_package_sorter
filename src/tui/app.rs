use crate::{backend, db, models::Package, models::SortKey};
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
}

pub struct App {
    pub packages: Vec<Package>,
    pub selected_package: ListState,
    pub sort_key: SortKey,
    pub filter_repo: Option<String>,
    pub filter_tag: Option<String>,
    pub input: String,
    pub input_mode: InputMode,
    pub show_explicit: bool,
    pub show_dependency: bool,
    pub output: Vec<String>,
    pub all_tags: Vec<String>,
    pub tag_selection: ListState,
}

impl App {
    pub fn new() -> Self {
        App {
            packages: Vec::new(),
            selected_package: ListState::default(),
            sort_key: SortKey::Name,
            filter_repo: None,
            filter_tag: None,
            input: String::new(),
            input_mode: InputMode::Normal,
            show_explicit: false,
            show_dependency: false,
            output: Vec::new(),
            all_tags: db::get_all_tags().unwrap_or_default(),
            tag_selection: ListState::default(),
        }
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> std::io::Result<()> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            self.packages = backend::get_all_packages().await.unwrap_or_default();
        });

        if !self.packages.is_empty() {
            self.selected_package.select(Some(0));
        }

        loop {
            // The call must be `ui::ui` because we imported the `ui` module
            terminal.draw(|f| ui::ui(f, self))?;
            // Now we can call `handle_events` directly
            if handle_events(self)? {
                break;
            }
        }
        Ok(())
    }

    pub fn sort_by_size(&mut self) {
        self.sort_key = SortKey::Size;
        backend::sort_packages(&mut self.packages, self.sort_key);
    }

    pub fn sort_by_name(&mut self) {
        self.sort_key = SortKey::Name;
        backend::sort_packages(&mut self.packages, self.sort_key);
    }

    pub fn sort_by_install_date(&mut self) {
        self.sort_key = SortKey::InstallDate;
        backend::sort_packages(&mut self.packages, self.sort_key);
    }

    pub fn select_previous(&mut self) {
        let i = match self.selected_package.selected() {
            Some(i) => {
                if i == 0 {
                    self.packages.len() - 1
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
                if i >= self.packages.len() - 1 {
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
        let i = match self.tag_selection.selected() {
            Some(i) => {
                if i == 0 {
                    self.all_tags.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.tag_selection.select(Some(i));
    }

    pub fn select_next_tag(&mut self) {
        let i = match self.tag_selection.selected() {
            Some(i) => {
                if i >= self.all_tags.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.tag_selection.select(Some(i));
    }

    pub fn reload_tags(&mut self) {
        self.all_tags = db::get_all_tags().unwrap_or_default();
    }
}