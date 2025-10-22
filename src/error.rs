use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON Serialization/Deserialization Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    #[error("Failed to parse package info: {0}")]
    ParseError(String),
    #[error("Invalid user input: {0}")]
    InvalidInput(String),
    #[error("Network request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("TOML Parse Error: {0}")] 
    TomlParse(String),
    #[error("TOML Serialize Error: {0}")] 
    TomlSerialize(String),
}