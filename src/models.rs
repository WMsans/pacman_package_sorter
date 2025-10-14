use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub repository: Repository,
    pub install_date: DateTime<Utc>,
    pub build_date: DateTime<Utc>,
    pub size: f64,
    pub is_explicit: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    pub popularity: Option<f64>,
    pub num_votes: Option<u32>,
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

impl From<&str> for Repository {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "core" => Self::Core,
            "extra" => Self::Extra,
            "multilib" => Self::Multilib,
            "community" => Self::Community,
            "aur" | "local" => Self::AUR,
            _ => Self::AUR,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortKey {
    Name,
    Size,
    InstallDate,
    UpdateDate,
    Popularity,
}

impl fmt::Display for SortKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SortKey::Name => write!(f, "Name"),
            SortKey::Size => write!(f, "Size"),
            SortKey::InstallDate => write!(f, "Installed Date"),
            SortKey::UpdateDate => write!(f, "Update Date"),
            SortKey::Popularity => write!(f, "Popularity"),
        }
    }
}

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