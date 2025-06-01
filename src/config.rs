use serde::{self, Deserialize, Deserializer};
use console::Color;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default, deserialize_with = "deserialize_console_color")]
    pub accent_color: Option<console::Color>,
    pub art: Option<ArtConfig>,
    pub user_host: Option<bool>,
    pub cpu: Option<bool>,
    pub memory: Option<bool>,
    pub uptime: Option<bool>,
    pub os: Option<bool>,
    pub kernel: Option<bool>,
    pub disk: Option<bool>,
    pub ip: Option<bool>,
    pub packages: Option<bool>,
    pub shell: Option<bool>,
    pub wm: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ArtConfig {
    pub max_length: Option<usize>,
    pub max_breadth: Option<usize>,
    pub fractal: Option<FractalType>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FractalType {
    MandelbrotSet,
    JuliaSet,
    None,
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        let contents = fs::read_to_string(&config_path)
            .unwrap_or_else(|_| panic!("Could not find config at: {:?}", config_path));
        toml::from_str(&contents).expect("Could not parse config file.")
    }

    fn get_config_path() -> PathBuf {
        dirs::config_dir()
            .expect("No config directory found")
            .join("kensaku/config.toml")
    }
}

fn deserialize_console_color<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    let color = match raw.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        s if s.starts_with("color256(") && s.ends_with(')') => {
            let inner = &s[9..s.len() - 1];
            let val: u8 = inner.parse().map_err(serde::de::Error::custom)?;
            Color::Color256(val)
        }
        _ => return Err(serde::de::Error::custom("Invalid color")),
    };
    Ok(Some(color))
}
