use crate::{
    db,
    error::AppError,
    packages::{
        aur::fetch_aur_package_data,
        models::{Package, Repository},
    },
};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use std::collections::HashMap;
use std::process::Command;

// Main function to get all installed packages
pub async fn get_all_packages() -> Result<Vec<Package>, AppError> {
    // Build a map of package names to their repositories for faster lookup
    let repo_map = build_repo_map()?;

    let output = Command::new("pacman")
        .arg("-Qi")
        .env("LC_ALL", "C") // Set locale to C to ensure consistent output format
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to execute pacman: {}", e)))?;

    if !output.status.success() {
        return Err(AppError::CommandFailed(
            "pacman -Qi command failed".to_string(),
        ));
    }

    let output_str = String::from_utf8(output.stdout)
        .map_err(|_| AppError::ParseError("Pacman output is not valid UTF-8".to_string()))?;

    // Load custom tags from our DB
    let tags_db = db::load_tags()?;

    // Each package info block is separated by a double newline
    let mut packages: Vec<Package> = output_str
        .trim()
        .split("\n\n")
        .filter_map(|block| parse_package_block(block, &tags_db, &repo_map).ok())
        .collect();

    // Get AUR packages and fetch their popularity data
    let aur_package_names: Vec<String> = packages
        .iter()
        .filter(|p| p.repository == Repository::AUR)
        .map(|p| p.name.clone())
        .collect();

    if !aur_package_names.is_empty() {
        let aur_data = fetch_aur_package_data(aur_package_names).await?;
        for pkg in &mut packages {
            if pkg.repository == Repository::AUR {
                if let Some(aur_pkg) = aur_data.get(&pkg.name) {
                    pkg.popularity = Some(aur_pkg.popularity);
                    pkg.num_votes = Some(aur_pkg.num_votes);
                }
            }
        }
    }

    Ok(packages)
}

/// Gets all available packages from repositories using `pacman -Sl`
pub fn get_all_available_packages() -> Result<Vec<Package>, AppError> {
    let output = Command::new("pacman")
        .arg("-Sl")
        .env("LC_ALL", "C")
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to execute pacman -Sl: {}", e)))?;

    if !output.status.success() {
        return Err(AppError::CommandFailed(
            "pacman -Sl command failed".to_string(),
        ));
    }

    let output_str = String::from_utf8(output.stdout)
        .map_err(|_| AppError::ParseError("pacman -Sl output is not valid UTF-8".to_string()))?;

    let mut packages = Vec::new();
    let now = Utc::now(); // For dummy dates

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        // Format: <repo> <name> <version> [installed_version]
        if parts.len() >= 3 {
            let repo = parts[0];
            let name = parts[1];
            let version = parts[2];

            let pkg = Package {
                name: name.to_string(),
                version: version.to_string(),
                description: "N/A (available package)".to_string(), // -Sl doesn't provide description
                repository: Repository::from(repo),
                install_date: now, // Dummy data
                build_date: now,   // Dummy data
                size: 0.0,         // Dummy data
                is_explicit: false, // Dummy data
                tags: Vec::new(),  // Not applicable
                popularity: None,  // Not fetched for this view
                num_votes: None,
            };
            packages.push(pkg);
        }
    }
    Ok(packages)
}

// Gets a list of orphan package names
pub fn get_orphan_package_names() -> Result<Vec<String>, AppError> {
    let output = Command::new("pacman")
        .arg("-Qdt")
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to execute pacman -Qdt: {}", e)))?;

    if !output.status.success() {
        // -Qdt can return 1 if no packages are found, which is not an error
        if output.stdout.is_empty() {
            return Ok(Vec::new());
        }
        return Err(AppError::CommandFailed(
            "pacman -Qdt command failed".to_string(),
        ));
    }

    let output_str = String::from_utf8(output.stdout)
        .map_err(|_| AppError::ParseError("pacman -Qdt output is not valid UTF-8".to_string()))?;

    let names: Vec<String> = output_str
        .lines()
        .filter_map(|line| line.split_whitespace().next().map(String::from))
        .collect();
    
    Ok(names)
}


// Builds a HashMap mapping package names to their repository
fn build_repo_map() -> Result<HashMap<String, String>, AppError> {
    let output = Command::new("pacman")
        .arg("-Sl")
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to execute pacman -Sl: {}", e)))?;

    if !output.status.success() {
        return Err(AppError::CommandFailed(
            "pacman -Sl command failed".to_string(),
        ));
    }

    let output_str = String::from_utf8(output.stdout)
        .map_err(|_| AppError::ParseError("pacman -Sl output is not valid UTF-8".to_string()))?;

    let mut repo_map = HashMap::new();
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let repo = parts[0];
            let name = parts[1];
            repo_map.insert(name.to_string(), repo.to_string());
        }
    }
    Ok(repo_map)
}

// Helper to parse a single package info block from `pacman -Qi`
fn parse_package_block(
    block: &str,
    tags_db: &std::collections::HashMap<String, Vec<String>>,
    repo_map: &HashMap<String, String>,
) -> Result<Package, AppError> {
    let mut fields = std::collections::HashMap::new();
    for line in block.lines() {
        if let Some((key, value)) = line.split_once(" : ") {
            fields.insert(key.trim(), value.trim());
        }
    }

    let name = fields
        .get("Name")
        .ok_or_else(|| AppError::ParseError("Missing Name".to_string()))?
        .to_string();

    // Try to get the repository from the pacman -Qi output,
    // otherwise fall back to the repo_map created from pacman -Sl.
    let repository_str = fields
        .get("Repository")
        .map(|s| s.to_string())
        .or_else(|| repo_map.get(&name).cloned())
        .unwrap_or_else(|| "Unknown".to_string());

    // Size parsing (e.g., "12.34 MiB" or "12,34 MiB")
    let size_str = fields.get("Installed Size").unwrap_or(&"0.0 KiB");
    let sanitized_size_str = size_str.replace(',', ".");
    let size_parts: Vec<&str> = sanitized_size_str.split_whitespace().collect();
    let size_val: f64 = size_parts.get(0).unwrap_or(&"0.0").parse().unwrap_or(0.0);
    let size_unit = size_parts.get(1).unwrap_or(&"MiB");
    let size_mib = match *size_unit {
        "GiB" => size_val * 1024.0,
        "MiB" => size_val,
        "KiB" => size_val / 1024.0,
        "B" => size_val / (1024.0 * 1024.0),
        _ => size_val,
    };

    // Date parsing
    let install_date_str = fields
        .get("Install Date")
        .ok_or_else(|| AppError::ParseError("Missing Install Date".to_string()))?;
    let build_date_str = fields
        .get("Build Date")
        .ok_or_else(|| AppError::ParseError("Missing Build Date".to_string()))?;

    let package = Package {
        name: name.clone(),
        version: fields.get("Version").unwrap_or(&"").to_string(),
        description: fields.get("Description").unwrap_or(&"").to_string(),
        repository: Repository::from(repository_str.as_str()),
        install_date: parse_pacman_date(install_date_str)?,
        build_date: parse_pacman_date(build_date_str)?,
        size: size_mib,
        is_explicit: fields.get("Install Reason").unwrap_or(&"") == &"Explicitly installed",
        tags: tags_db.get(&name).cloned().unwrap_or_default(),
        popularity: None,
        num_votes: None,
    };
    Ok(package)
}
// Helper to parse pacman's date format
fn parse_pacman_date(date_str: &str) -> Result<DateTime<Utc>, AppError> {
    // Attempt to parse the date with a few different formats
    let formats = [
        "%a %b %d %H:%M:%S %Y",       // "Wed May 01 21:30:00 2024" (LC_ALL=C format)
        "%a %d %b %Y %I:%M:%S %p %Z", // "Wed 01 May 2024 09:30:00 PM UTC"
    ];

    for fmt in formats.iter() {
        if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, fmt) {
            return Ok(Utc.from_utc_datetime(&dt));
        }
    }

    // Fallback for formats with a timezone name, assuming local time
    if let Ok(dt) = DateTime::parse_from_str(date_str, "%a %d %b %Y %I:%M:%S %p %Z") {
        return Ok(dt.with_timezone(&Utc));
    }

    // If all parsing fails, print a warning and return the current time as a fallback.
    eprintln!(
        "Warning: Failed to parse date '{}'. Using current time as a fallback.",
        date_str
    );
    Ok(Utc::now())
}