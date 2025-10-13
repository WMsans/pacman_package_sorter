use crate::error::AppError;
use serde_json;
use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::PathBuf,
};

// Type alias for our tag database
type TagDb = HashMap<String, Vec<String>>;

// Function to get the path to our tags.json file
fn get_db_path() -> Result<PathBuf, AppError> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "Config directory not found")))?
        .join("pacman_package_sorter");
    
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
pub fn add_tag(package_name: &str, tag: &str) -> Result<String, AppError> {
    let mut db = load_tags()?;
    let tags = db.entry(package_name.to_string()).or_insert_with(Vec::new);
    if !tags.contains(&tag.to_string()) {
        tags.push(tag.to_string());
        tags.sort(); // Keep tags sorted
    }
    save_tags(&db)?;
    Ok(format!("Added tag '{}' to '{}'", tag, package_name))
}

// Removes a tag from a package
pub fn remove_tag(package_name: &str, tag: &str) -> Result<String, AppError> {
    let mut db = load_tags()?;
    let mut changed = false;
    let mut is_empty = false;

    if let Some(tags) = db.get_mut(package_name) {
        let original_len = tags.len();
        tags.retain(|t| t != tag);
        changed = tags.len() < original_len;
        is_empty = tags.is_empty();
    }

    if is_empty {
        db.remove(package_name);
    }

    if changed {
        save_tags(&db)?;
        Ok(format!("Removed tag '{}' from '{}'", tag, package_name))
    } else {
        Err(AppError::InvalidInput(format!("Tag '{}' not found for package '{}'.", tag, package_name)))
    }
}

// Get all unique tags
pub fn get_all_tags() -> Result<Vec<String>, AppError> {
    let db = load_tags()?;
    let all_tags: BTreeSet<String> = db.into_values().flatten().collect();
    Ok(all_tags.into_iter().collect())
}