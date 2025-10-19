use crate::{
    backend, db,
    packages::{models::{Package}, pacman}, 
};

// --- Enums for application state ---

pub enum InputMode {
    Normal,
    Tagging,
    Untagging,
    Sorting,
    Filtering,
    Searching,
    Showing, 
}

pub enum FilterFocus {
    Search,
    Tags,
    Repos,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TagModalFocus {
    Input,
    List,
}
// --- State Management Structs ---

/// Holds the core data of the application
pub struct AppState {
    pub packages: Vec<Package>,
    pub available_packages: Vec<Package>, 
    pub filtered_packages: Vec<Package>,
    pub all_tags: Vec<String>,
    pub all_repos: Vec<String>,
    pub orphan_package_names: Vec<String>, 
}

impl AppState {
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
            available_packages: Vec::new(), 
            filtered_packages: Vec::new(),
            all_tags: db::get_all_tags().unwrap_or_default(),
            all_repos: Vec::new(),
            orphan_package_names: Vec::new(), 
        }
    }

    /// Reloads packages and repos from the backend.
    pub async fn load_packages(&mut self) {
        self.packages = crate::packages::pacman::get_all_packages()
            .await
            .unwrap_or_default();
        
        self.available_packages = pacman::get_all_available_packages().unwrap_or_default();
        
        // Use the more complete list of available packages to build the repo list
        self.all_repos = backend::get_all_repos(&self.available_packages);
        
        self.orphan_package_names = pacman::get_orphan_package_names().unwrap_or_default(); 
    }
}