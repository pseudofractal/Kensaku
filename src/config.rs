use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub art: Option<String>,
    pub user_host: Option<bool>,
    pub cpu: Option<bool>,
    pub memory: Option<bool>,
    pub uptime: Option<bool>,
    pub os: Option<bool>,
    pub kernel: Option<bool>,
    pub disk: Option<bool>,
    pub ip: Option<bool>,
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
