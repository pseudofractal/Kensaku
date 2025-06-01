use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub ascii_art: Option<String>,
    pub show_cpu: Option<bool>,
    pub show_memory: Option<bool>,
    pub show_uptime: Option<bool>,
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        let contents = fs::read_to_string(&config_path)
            .unwrap_or_else(|_| panic!("Could not find config at: {:?}", config_path));
        toml::from_str(&contents).expect("Could not parse config file.")
    }

    fn get_config_path() -> PathBuf {
        dirs::config_dir().expect("No config directory found").join("kensaku/config.toml")
    }
}
