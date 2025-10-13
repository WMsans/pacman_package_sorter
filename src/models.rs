use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub repository: Repository,
    pub install_date: DateTime<Utc>,
    pub build_date: DateTime<Utc>, // Used for "last updated"
    pub size: f64, // Size in MiB
    pub is_explicit: bool,
    #[serde(default)] // Tags might not exist in the file
    pub tags: Vec<String>,
    // Add fields for AUR data
    pub popularity: Option<f64>,
    pub num_votes: Option<u32>,
    // TODO: Implement dependency size calculation
    // pub dependency_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Repository {
    Core,
    Extra,
    Multilib,
    Community,
    AUR,
    Unknown,
}

// Allows us to create a Repository from a string (e.g., "core")
impl From<&str> for Repository {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "core" => Self::Core,
            "extra" => Self::Extra,
            "multilib" => Self::Multilib,
            "community" => Self::Community,
            "aur" | "local" => Self::AUR, // pacman calls AUR pkgs 'local'
            _ => Self::AUR,
        }
    }
}

// Enum for our sorting options
#[derive(Debug, Clone, Copy)]
pub enum SortKey {
    Name,
    Size,
    InstallDate,
    UpdateDate,
    Popularity,
    // TODO: Add DependencySize
}

// Allows clap to parse this from a string argument
impl FromStr for SortKey {
    type Err = crate::error::AppError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "name" => Ok(Self::Name),
            "size" => Ok(Self::Size),
            "installed" => Ok(Self::InstallDate),
            "updated" => Ok(Self::UpdateDate),
            "popularity" => Ok(Self::Popularity),
            _ => Err(AppError::InvalidInput(format!("Invalid sort key: {}", s))),
        }
    }
}