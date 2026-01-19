use console::Color;
use serde::{Deserialize, Deserializer};
use std::{fs, io, path::PathBuf};

const DEFAULT_CONFIG: &str = r#"
accent_color = "cyan"
user_host = true
os = true
kernel = true
cpu = true
memory = true
disk = true
uptime = true
shell = true
[art]
fractal = "random"
max_length = 50
max_breadth = 20
"#;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
  #[serde(default, deserialize_with = "deserialize_console_color")]
  pub accent_color: Option<Color>,
  pub art: Option<ArtConfig>,
  pub user_host: Option<bool>,
  pub cpu: Option<bool>,
  pub memory: Option<bool>,
  pub swap: Option<bool>,
  pub uptime: Option<bool>,
  pub os: Option<bool>,
  pub kernel: Option<bool>,
  pub disk: Option<bool>,
  #[serde(alias = "network")]
  pub ip: Option<bool>,
  pub packages: Option<bool>,
  pub shell: Option<bool>,
  pub wm: Option<bool>,
  pub load: Option<bool>,
  pub cpu_threads: Option<bool>,
  pub terminal: Option<bool>,
  pub host_model: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtConfig {
  pub max_length: Option<usize>,
  pub max_breadth: Option<usize>,
  pub fractal: Option<FractalType>,
  pub seed: Option<u64>,
  pub depth: Option<u8>,
  pub julia_c: Option<JuliaParameter>,
  pub multibrot_power: Option<PowerParameter>,
  pub max_iterations: Option<usize>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum RandomParameter {
  Random,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(untagged)]
pub enum JuliaParameter {
  Fixed([f64; 2]),
  Random(RandomParameter),
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(untagged)]
pub enum PowerParameter {
  Fixed(u8),
  Random(RandomParameter),
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum FractalType {
  Random,
  SierpinskiTriangle,
  SierpinskiCarpet,
  CantorDust,
  Vicsek,
  KochCurve,
  MandelbrotSet,
  JuliaSet,
  BurningShip,
  Tricorn,
  Multibrot,
  None,
}

impl Config {
  pub fn load() -> Result<Self, String> {
    let path = Self::get_config_path()?;
    let contents = match fs::read_to_string(&path) {
      Ok(contents) => contents,
      Err(error) if error.kind() == io::ErrorKind::NotFound => DEFAULT_CONFIG.to_owned(),
      Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
    };
    let config: Self = toml::from_str(&contents)
      .map_err(|error| format!("invalid configuration in {}: {error}", path.display()))?;
    if let Some(art) = &config.art {
      art.validate()?;
    }
    Ok(config)
  }

  fn get_config_path() -> Result<PathBuf, String> {
    dirs::config_dir()
      .map(|path| path.join("kensaku/config.toml"))
      .ok_or_else(|| "cannot determine the configuration directory".into())
  }
}

impl ArtConfig {
  fn validate(&self) -> Result<(), String> {
    if self
      .max_length
      .is_some_and(|width| !(1..=500).contains(&width))
      || self
        .max_breadth
        .is_some_and(|height| !(1..=200).contains(&height))
    {
      return Err("art dimensions must be between 1 and 500 columns and 1 and 200 rows".into());
    }
    if self.depth.is_some_and(|depth| !(1..=6).contains(&depth)) {
      return Err("art depth must be between 1 and 6".into());
    }
    if let Some(JuliaParameter::Fixed(components)) = self.julia_c {
      if components.iter().any(|component| !component.is_finite()) {
        return Err("julia_c must contain two finite numbers".into());
      }
    }
    if let Some(PowerParameter::Fixed(power)) = self.multibrot_power {
      if !(2..=8).contains(&power) {
        return Err("multibrot_power must be between 2 and 8, or \"random\"".into());
      }
    }
    if self
      .max_iterations
      .is_some_and(|iterations| !(1..=1000).contains(&iterations))
    {
      return Err("max_iterations must be between 1 and 1000".into());
    }
    Ok(())
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
    s if s.starts_with("color256(") && s.ends_with(')') => Color::Color256(
      s[9..s.len() - 1]
        .parse()
        .map_err(serde::de::Error::custom)?,
    ),
    _ => return Err(serde::de::Error::custom("invalid color")),
  };
  Ok(Some(color))
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn defaults_and_legacy_network_parse() {
    assert!(toml::from_str::<Config>(DEFAULT_CONFIG).is_ok());
    let config: Config = toml::from_str("network = true").unwrap();
    assert_eq!(config.ip, Some(true));
    assert!(toml::from_str::<Config>("memor = true").is_err());
  }
}
