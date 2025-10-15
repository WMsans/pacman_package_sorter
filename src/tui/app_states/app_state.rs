use crate::{
    backend, db,
    packages::models::{Package},
};

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
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
            filtered_packages: Vec::new(),
            all_tags: db::get_all_tags().unwrap_or_default(),
            all_repos: Vec::new(),
        }
    }

    /// Reloads packages and repos from the backend.
    pub async fn load_packages(&mut self) {
        self.packages = crate::packages::pacman::get_all_packages()
            .await
            .unwrap_or_default();
        self.all_repos = backend::get_all_repos(&self.packages);
    }
}