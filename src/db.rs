use crate::error::AppError;
use serde_json;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
};

// Type alias for our tag database
type TagDb = HashMap<String, Vec<String>>;

// Function to get the path to our tags.json file
fn get_db_path() -> Result<PathBuf, AppError> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "Config directory not found")))?
        .join("pacman_plus");
    
    fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("tags.json"))
}

// Loads the tags from the JSON file
pub fn load_tags() -> Result<TagDb, AppError> {
    let path = get_db_path()?;
    if !path.exists() {
        return Ok(HashMap::new()); // Return empty map if file doesn't exist
    }
    let content = fs::read_to_string(path)?;
    let db = serde_json::from_str(&content)?;
    Ok(db)
}

// Saves the tags to the JSON file
fn save_tags(db: &TagDb) -> Result<(), AppError> {
    let path = get_db_path()?;
    let content = serde_json::to_string_pretty(db)?;
    fs::write(path, content)?;
    Ok(())
}

// Adds a tag to a package
pub fn add_tag(package_name: &str, tag: &str) -> Result<(), AppError> {
    let mut db = load_tags()?;
    let tags = db.entry(package_name.to_string()).or_insert_with(Vec::new);
    if !tags.contains(&tag.to_string()) {
        tags.push(tag.to_string());
        tags.sort(); // Keep tags sorted
    }
    save_tags(&db)?;
    println!("Added tag '{}' to '{}'", tag, package_name);
    Ok(())
}

// Removes a tag from a package
pub fn remove_tag(package_name: &str, tag: &str) -> Result<(), AppError> {
    let mut db = load_tags()?;
    if let Some(tags) = db.get_mut(package_name) {
        tags.retain(|t| t != tag);
        if tags.is_empty() {
            db.remove(package_name);
        }
        save_tags(&db)?;
        println!("Removed tag '{}' from '{}'", tag, package_name);
    } else {
        println!("Package '{}' not found in tag database.", package_name);
    }
    Ok(())
}