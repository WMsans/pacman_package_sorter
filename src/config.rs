use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

// --- Data Structures ---

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConfigKey {
    pub key: char,
    #[serde(default)]
    pub shift: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ActionType {
    /// Runs a shell command
    Command {
        command: Vec<String>,
        #[serde(default)]
        requires_package: bool,
        #[serde(default)]
        show_mode_whitelist: Vec<String>,
        #[serde(default)]
        show_mode_blacklist: Vec<String>,
    },
    /// Triggers an internal application action
    Local,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Action {
    pub name: String,
    pub key: ConfigKey,
    #[serde(flatten)]
    pub action_type: ActionType,
}

impl Action {
    /// Helper for creating internal actions for the action modal
    pub fn new_local(name: &str, key: char, shift: bool) -> Self {
        Self {
            name: name.to_string(),
            key: ConfigKey { key, shift },
            action_type: ActionType::Local,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub actions: Vec<Action>,
}

impl Default for Config {
    /// Defines the default actions, replacing the old hardcoded keys
    fn default() -> Self {
        Self {
            actions: vec![
                Action {
                    name: "System Upgrade (pacman)".to_string(),
                    key: ConfigKey { key: 'S', shift: true },
                    action_type: ActionType::Command {
                        command: vec!["sudo".to_string(), "pacman".to_string(), "-Syu".to_string()],
                        requires_package: false,
                        show_mode_whitelist: vec![],
                        show_mode_blacklist: vec![],
                    },
                },
                Action {
                    name: "System Upgrade (yay)".to_string(),
                    key: ConfigKey { key: 'Y', shift: true },
                    action_type: ActionType::Command {
                        command: vec!["yay".to_string(), "-Syu".to_string()],
                        requires_package: false,
                        show_mode_whitelist: vec![],
                        show_mode_blacklist: vec![],
                    },
                },
                Action {
                    name: "Install Package".to_string(),
                    key: ConfigKey { key: 'i', shift: false },
                    action_type: ActionType::Command {
                        command: vec!["sudo".to_string(), "pacman".to_string(), "-S".to_string(), "{package}".to_string()],
                        requires_package: true,
                        show_mode_whitelist: vec!["All Available".to_string()],
                        show_mode_blacklist: vec![],
                    },
                },
                Action {
                    name: "Uninstall Package".to_string(),
                    key: ConfigKey { key: 'u', shift: false },
                    action_type: ActionType::Command {
                        command: vec!["sudo".to_string(), "pacman".to_string(), "-Rns".to_string(), "{package}".to_string()],
                        requires_package: true,
                        show_mode_whitelist: vec![],
                        show_mode_blacklist: vec!["All Available".to_string()],
                    },
                },
                Action {
                    name: "Remove Orphan Packages".to_string(),
                    key: ConfigKey { key: 'o', shift: false },
                    action_type: ActionType::Command {
                        command: vec![
                            "sudo".to_string(),
                            "sh".to_string(),
                            "-c".to_string(),
                            "pacman -Rns $(pacman -Qdtq)".to_string(),
                        ],
                        requires_package: false,
                        show_mode_whitelist: vec![],
                        show_mode_blacklist: vec![],
                    },
                },
            ],
        }
    }
}

// --- Config Loading ---

fn get_config_dir() -> Result<PathBuf, AppError> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "Config directory not found")))?
        .join("pacman_package_sorter");

    fs::create_dir_all(&config_dir)?;
    Ok(config_dir)
}

fn get_config_path() -> Result<PathBuf, AppError> {
    Ok(get_config_dir()?.join("config.toml"))
}

/// Loads the config from file, or creates and returns a default config.
pub fn load_config() -> Config {
    let path = match get_config_path() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to get config path: {}. Using default config.", e);
            return Config::default();
        }
    };

    if !path.exists() {
        let default_config = Config::default();
        let toml_string =
            toml::to_string_pretty(&default_config).expect("Failed to serialize default config");

        if let Err(e) = fs::write(&path, toml_string) {
            eprintln!("Failed to write default config to {:?}: {}", path, e);
        }
        return default_config;
    }

    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!(
                    "Failed to parse config file at {:?}: {}. Using default config.",
                    path, e
                );
                Config::default()
            }
        },
        Err(e) => {
            eprintln!(
                "Failed to read config file at {:?}: {}. Using default config.",
                path, e
            );
            Config::default()
        }
    }
}

/// Replaces placeholders in a command template with dynamic values.
pub fn template_command(
    command_template: &[String],
    package_name: Option<&str>,
) -> Result<Vec<String>, AppError> {
    let final_command = command_template
        .iter()
        .map(|part| {
            if part == "{package}" {
                package_name.unwrap_or("{package}").to_string()
            } else {
                part.clone()
            }
        })
        .collect::<Vec<String>>();

    Ok(final_command)
}