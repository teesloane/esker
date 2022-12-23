use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub url: String,
    pub title: String,

    // Optional fields
    pub attachment_directory: Option<String>,
    pub description: Option<String>,
    pub ignored_directories: Option<Vec<String>>,
}

impl Config {
    /// Load the config from a yaml file and get return the Config struct.
    pub fn new(cwd: &PathBuf) -> Config {
        let config_file = cwd.join("_esker/config.yaml");
        let user_config_str = std::fs::read_to_string(&config_file).expect("Failed to load user config.yaml");
        let user_config: Config = serde_yaml::from_str(&user_config_str).expect("Invalid yaml found in config.yaml");
        return user_config;
    }
}
