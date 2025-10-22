use crate::{
    db,
    packages::{models::{Package}}, 
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
    Action,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActionModalFocus{
    Input,
    List,
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
pub struct LoadedData {
    pub packages: Vec<Package>,
    pub available_packages: Vec<Package>, 
    pub all_repos: Vec<String>,
    pub orphan_package_names: Vec<String>, 
}
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
}