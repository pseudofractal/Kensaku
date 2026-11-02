use crate::domain::{self, Complex, SetOp, Token};
use console::Color;
use serde::{Deserialize, Deserializer};
use std::{fs, io, ops::RangeInclusive, path::PathBuf};

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

/// Integers accepted by the Multibrot exponent.
pub const MULTIBROT_POWER_RANGE: RangeInclusive<u8> = 2..=8;

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
  pub fractal: Option<FractalName>,
  pub seed: Option<u64>,
  pub depth: Option<u8>,
  pub max_iterations: Option<usize>,
  pub julia: Option<JuliaConfig>,
  pub multibrot: Option<MultibrotConfig>,
}

/// Parameters for the Julia set, read from `[art.julia]`.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct JuliaConfig {
  pub c: Option<Param<Complex>>,
}

/// Parameters for the Multibrot set, read from `[art.multibrot]`.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct MultibrotConfig {
  pub power: Option<Param<u8>>,
}

/// The value of a fractal parameter: a constant, the parameter's standard
/// random domain, an explicit domain, or a finite set to pick from.
///
/// Deserialized by hand (see `parse_param`) so a malformed value reports the
/// expected form instead of serde's generic untagged-enum message.
#[derive(Debug, Clone)]
pub enum Param<T> {
  Fixed(T),
  Random,
  Range([T; 2]),
  Fold(Vec<Token<T>>),
  OneOf { one_of: Vec<T> },
}

/// The fractal a run draws, or the `random`/`none` selectors.
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum FractalName {
  Random,
  None,
  Mandelbrot,
  Julia,
  BurningShip,
  Tricorn,
  Multibrot,
  SierpinskiTriangle,
  SierpinskiCarpet,
  CantorDust,
  Vicsek,
  KochCurve,
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
    if self
      .max_iterations
      .is_some_and(|iterations| !(1..=1000).contains(&iterations))
    {
      return Err("max_iterations must be between 1 and 1000".into());
    }
    self.validate_parameter_selection()?;
    if let Some(julia) = &self.julia {
      julia.validate()?;
    }
    if let Some(multibrot) = &self.multibrot {
      multibrot.validate()?;
    }
    Ok(())
  }

  fn validate_parameter_selection(&self) -> Result<(), String> {
    let Some(fractal) = self.fractal else {
      if self.julia.is_some() || self.multibrot.is_some() {
        return Err(
          "fractal parameters were specified but no fractal is selected; set `fractal`".into(),
        );
      }
      return Ok(());
    };
    if matches!(fractal, FractalName::Random) {
      return Ok(());
    }
    if self.julia.is_some() && !matches!(fractal, FractalName::Julia) {
      return Err(parameter_selection_error("julia", fractal));
    }
    if self.multibrot.is_some() && !matches!(fractal, FractalName::Multibrot) {
      return Err(parameter_selection_error("multibrot", fractal));
    }
    Ok(())
  }
}

fn parameter_selection_error(parameter: &str, selected: FractalName) -> String {
  match selected {
    FractalName::Julia => {
      format!("{parameter} parameters were specified but the julia fractal is selected")
    }
    FractalName::Multibrot => {
      format!("{parameter} parameters were specified but the multibrot fractal is selected")
    }
    _ => {
      format!("{parameter} parameters were specified but the {parameter} fractal is not selected")
    }
  }
}

impl JuliaConfig {
  fn validate(&self) -> Result<(), String> {
    if let Some(c) = &self.c {
      validate_complex_param(c)?;
    }
    Ok(())
  }
}

impl MultibrotConfig {
  fn validate(&self) -> Result<(), String> {
    if let Some(power) = &self.power {
      validate_power_param(power)?;
    }
    Ok(())
  }
}

fn validate_complex_param(param: &Param<Complex>) -> Result<(), String> {
  let finite = |value: &Complex| value.iter().all(|part| part.is_finite());
  let ordered = |from: &Complex, to: &Complex| from[0] <= to[0] && from[1] <= to[1];
  match param {
    Param::Fixed(value) => {
      if !finite(value) {
        return Err("julia c must contain finite numbers".into());
      }
    }
    Param::Random => {}
    Param::Range([from, to]) => {
      if !finite(from) || !finite(to) {
        return Err("julia c range must contain finite numbers".into());
      }
      if !ordered(from, to) {
        return Err("julia c range start must not exceed its end".into());
      }
    }
    Param::Fold(tokens) => {
      validate_fold(
        tokens,
        finite,
        "julia c must contain finite numbers",
        ordered,
      )?;
      if domain::complex_domain_is_empty(tokens) {
        return Err("julia c domain is empty".into());
      }
    }
    Param::OneOf { one_of } => {
      if one_of.is_empty() {
        return Err("julia c set must not be empty".into());
      }
      if one_of.iter().any(|value| !finite(value)) {
        return Err("julia c set must contain finite numbers".into());
      }
    }
  }
  Ok(())
}

fn multibrot_power_error() -> String {
  format!(
    "multibrot power must be an integer between {} and {}",
    MULTIBROT_POWER_RANGE.start(),
    MULTIBROT_POWER_RANGE.end()
  )
}

fn validate_power_param(param: &Param<u8>) -> Result<(), String> {
  let in_domain = |value: &u8| MULTIBROT_POWER_RANGE.contains(value);
  let ordered = |from: &u8, to: &u8| from <= to;
  match param {
    Param::Fixed(value) => {
      if !in_domain(value) {
        return Err(multibrot_power_error());
      }
    }
    Param::Random => {}
    Param::Range([from, to]) => {
      if !in_domain(from) || !in_domain(to) {
        return Err(multibrot_power_error());
      }
      if !ordered(from, to) {
        return Err("multibrot power range start must not exceed its end".into());
      }
    }
    Param::Fold(tokens) => {
      validate_fold(
        tokens,
        |value: &u8| MULTIBROT_POWER_RANGE.contains(value),
        &multibrot_power_error(),
        ordered,
      )?;
      if domain::power_domain_is_empty(tokens) {
        return Err("multibrot power domain is empty".into());
      }
    }
    Param::OneOf { one_of } => {
      if one_of.is_empty() {
        return Err("multibrot power set must not be empty".into());
      }
      if one_of.iter().any(|value| !in_domain(value)) {
        return Err(multibrot_power_error());
      }
    }
  }
  Ok(())
}

fn validate_fold<T>(
  tokens: &[Token<T>],
  check: impl Fn(&T) -> bool,
  check_message: &str,
  ordered: impl Fn(&T, &T) -> bool,
) -> Result<(), String> {
  if tokens.len() < 3 || tokens.len().is_multiple_of(2) {
    return Err(
      "domain must be `range op range ...` with an odd number of terms, at least three".into(),
    );
  }
  for (index, token) in tokens.iter().enumerate() {
    let expects_range = index % 2 == 0;
    match (expects_range, token) {
      (true, Token::Range([from, to])) => {
        if !check(from) || !check(to) {
          return Err(check_message.into());
        }
        if !ordered(from, to) {
          return Err("domain range start must not exceed its end".into());
        }
      }
      (false, Token::Op(_)) => {}
      _ => {
        return Err(
          "domain must alternate range then set operation, starting and ending with a range".into(),
        )
      }
    }
  }
  Ok(())
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

const SET_OPERATION_LIST: &str = "\"union\", \"intersection\", or \"difference\"";
const DOMAIN_EXAMPLE: &str = "[range, \"union\"|\"intersection\"|\"difference\", range, ...]";

/// The TOML spellings of one parameter type's values, used in error messages.
struct ParamForms {
  /// One value of the parameter, e.g. "[real, imaginary]".
  value: String,
  /// A single range of the parameter, e.g. "[a, b]".
  range: &'static str,
}

fn complex_forms() -> ParamForms {
  ParamForms {
    value: "[real, imaginary]".to_string(),
    range: "[[real, imaginary], [real, imaginary]]",
  }
}

fn power_forms() -> ParamForms {
  ParamForms {
    value: format!(
      "an integer between {} and {}",
      MULTIBROT_POWER_RANGE.start(),
      MULTIBROT_POWER_RANGE.end()
    ),
    range: "[a, b]",
  }
}

impl<'de> Deserialize<'de> for Param<Complex> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    parse_param(
      &toml::Value::deserialize(deserializer)?,
      &complex_forms(),
      complex_value,
    )
    .map_err(serde::de::Error::custom)
  }
}

impl<'de> Deserialize<'de> for Param<u8> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    parse_param(
      &toml::Value::deserialize(deserializer)?,
      &power_forms(),
      power_value,
    )
    .map_err(serde::de::Error::custom)
  }
}

/// Interpret one raw TOML value as a parameter, trying the forms in the same
/// order the renderer treats them: constant, "random", range, domain, set.
fn parse_param<T>(
  raw: &toml::Value,
  forms: &ParamForms,
  parse_value: fn(&toml::Value) -> Option<T>,
) -> Result<Param<T>, String> {
  if let Some(value) = parse_value(raw) {
    return Ok(Param::Fixed(value));
  }
  match raw {
    toml::Value::String(keyword) => {
      if keyword == "random" {
        Ok(Param::Random)
      } else {
        Err("the only accepted string is \"random\"".into())
      }
    }
    toml::Value::Array(items) => {
      if let Some([from, to]) = parse_range(items, parse_value) {
        return Ok(Param::Range([from, to]));
      }
      if let Some(tokens) = items
        .iter()
        .map(|item| parse_token(item, parse_value))
        .collect::<Option<Vec<_>>>()
      {
        return Ok(Param::Fold(tokens));
      }
      if let Some(word) = items
        .iter()
        .find_map(|item| item.as_str().filter(|word| set_op(word).is_none()))
      {
        return Err(format!(
          "unknown set operation \"{word}\"; expected {SET_OPERATION_LIST}"
        ));
      }
      if items.iter().any(|item| item.as_str().is_some()) {
        return Err(format!("each domain range must look like {}", forms.range));
      }
      Err(format!(
        "a bare list must be a range {} or a domain {DOMAIN_EXAMPLE}; to choose one of several values, use {{ one_of = [...] }}",
        forms.range
      ))
    }
    toml::Value::Table(table) => {
      let entries = table
        .get("one_of")
        .ok_or_else(|| "a table parameter must use { one_of = [...] }".to_string())?;
      if let Some(key) = table.keys().find(|key| key.as_str() != "one_of") {
        return Err(format!(
          "a table parameter accepts only `one_of`; unexpected key \"{}\"",
          key.as_str()
        ));
      }
      let items = entries
        .as_array()
        .ok_or_else(|| "one_of must be a list".to_string())?;
      let one_of = items
        .iter()
        .map(parse_value)
        .collect::<Option<Vec<T>>>()
        .ok_or_else(|| format!("each one_of entry must be {}", forms.value))?;
      Ok(Param::OneOf { one_of })
    }
    _ => Err(format!(
      "expected {}, \"random\", a range {}, a domain {DOMAIN_EXAMPLE}, or {{ one_of = [...] }}",
      forms.value, forms.range
    )),
  }
}

/// One term of a domain expression: a `[from, to]` range or a set operation.
fn parse_token<T>(
  raw: &toml::Value,
  parse_value: fn(&toml::Value) -> Option<T>,
) -> Option<Token<T>> {
  if let Some(word) = raw.as_str() {
    return set_op(word).map(Token::Op);
  }
  let items = raw.as_array()?;
  parse_range(items, parse_value).map(Token::Range)
}

/// A `[from, to]` pair of values, the shape of both a range and a domain term.
fn parse_range<T>(
  items: &[toml::Value],
  parse_value: fn(&toml::Value) -> Option<T>,
) -> Option<[T; 2]> {
  if items.len() != 2 {
    return None;
  }
  Some([parse_value(&items[0])?, parse_value(&items[1])?])
}

fn set_op(word: &str) -> Option<SetOp> {
  match word {
    "union" => Some(SetOp::Union),
    "intersection" => Some(SetOp::Intersection),
    "difference" => Some(SetOp::Difference),
    _ => None,
  }
}

/// A `[real, imaginary]` pair; TOML integers are accepted as reals.
fn complex_value(raw: &toml::Value) -> Option<Complex> {
  let items = raw.as_array()?;
  if items.len() != 2 {
    return None;
  }
  Some([number(&items[0])?, number(&items[1])?])
}

fn number(raw: &toml::Value) -> Option<f64> {
  match raw {
    toml::Value::Integer(value) => Some(*value as f64),
    toml::Value::Float(value) => Some(*value),
    _ => None,
  }
}

fn power_value(raw: &toml::Value) -> Option<u8> {
  match raw {
    toml::Value::Integer(power) => u8::try_from(*power).ok(),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn parse(config: &str) -> Result<ArtConfig, String> {
    let art: ArtConfig = toml::from_str(config).map_err(|error| error.to_string())?;
    art.validate()?;
    Ok(art)
  }

  #[test]
  fn defaults_and_legacy_network_parse() {
    assert!(toml::from_str::<Config>(DEFAULT_CONFIG).is_ok());
    let config: Config = toml::from_str("network = true").unwrap();
    assert_eq!(config.ip, Some(true));
    assert!(toml::from_str::<Config>("memor = true").is_err());
  }

  #[test]
  fn accepts_parameter_modes() {
    for config in [
      "fractal = \"julia\"\njulia = { c = [-0.8, 0.156] }",
      "fractal = \"julia\"\njulia = { c = \"random\" }",
      "fractal = \"julia\"\njulia = { c = [[-1.0, -1.0], [1.0, 1.0]] }",
      "fractal = \"julia\"\njulia = { c = [[[-1.0, -1.0], [1.0, 1.0]], \"difference\", [[-0.25, -0.25], [0.25, 0.25]]] }",
      "fractal = \"julia\"\njulia = { c = { one_of = [[-0.8, 0.156], [0.285, 0.01]] } }",
      "fractal = \"multibrot\"\nmultibrot = { power = [[2, 3], \"union\", [6, 8]] }",
      "fractal = \"multibrot\"\nmultibrot = { power = { one_of = [3, 5, 7] } }",
      "fractal = \"random\"\njulia = { c = [0.0, 0.0] }",
    ] {
      assert!(parse(config).is_ok(), "should accept: {config}");
    }
  }

  #[test]
  fn rejects_malformed_domains() {
    assert!(parse("fractal = \"julia\"\njulia = { c = [[-1.0, -1.0]] }").is_err());
    assert!(
      parse("fractal = \"julia\"\njulia = { c = [[[0.0, 0.0], [1.0, 1.0]], \"union\"] }").is_err()
    );
    assert!(parse(
      "fractal = \"julia\"\njulia = { c = [[[0.0, 0.0], [1.0, 1.0]], \"union\", [[0.0, 0.0], [0.5, 0.5]], \"union\"] }"
    )
    .is_err());
    assert!(parse("fractal = \"multibrot\"\nmultibrot = { power = { one_of = [] } }").is_err());
    assert!(parse("fractal = \"multibrot\"\nmultibrot = { power = 9 }").is_err());
    assert!(
      parse("fractal = \"multibrot\"\nmultibrot = { power = [[8, 2], \"union\", [6, 8]] }")
        .is_err()
    );
  }

  #[test]
  fn rejects_parameters_without_matching_selection() {
    assert!(parse("fractal = \"mandelbrot\"\njulia = { c = [0.0, 0.0] }").is_err());
    assert!(parse("julia = { c = [0.0, 0.0] }").is_err());
    assert!(parse("fractal = \"random\"\njulia = { c = [0.0, 0.0] }").is_ok());
    let error = parse("fractal = \"julia\"\nmultibrot = { power = 3 }").unwrap_err();
    assert!(
      error.contains("multibrot parameters were specified but the julia fractal is selected"),
      "got: {error}"
    );
  }

  #[test]
  fn invalid_parameter_shapes_explain_the_expected_form() {
    for (config, message) in [
      (
        "fractal = \"multibrot\"\nmultibrot = { power = [3, 5, 7] }",
        "use { one_of = [...] }",
      ),
      (
        "fractal = \"julia\"\njulia = { c = [[-0.8, 0.156], [0.285, 0.01], [0.0, 0.0]] }",
        "use { one_of = [...] }",
      ),
      (
        "fractal = \"multibrot\"\nmultibrot = { power = [[2, 3], \"xor\", [6, 8]] }",
        "unknown set operation \"xor\"",
      ),
      (
        "fractal = \"julia\"\njulia = { c = [0.1] }",
        "a bare list must be a range",
      ),
      (
        "fractal = \"multibrot\"\nmultibrot = { power = 3.0 }",
        "expected an integer",
      ),
      (
        "fractal = \"multibrot\"\nmultibrot = { power = -1 }",
        "an integer between 2 and 8",
      ),
      (
        "fractal = \"multibrot\"\nmultibrot = { power = 300 }",
        "an integer between 2 and 8",
      ),
      (
        "fractal = \"multibrot\"\nmultibrot = { power = { one_of = [-1] } }",
        "an integer between 2 and 8",
      ),
      (
        "fractal = \"julia\"\njulia = { c = \"fixed\" }",
        "the only accepted string is \"random\"",
      ),
      (
        "fractal = \"julia\"\njulia = { c = { from = [-1, -1], to = [1, 1] } }",
        "a table parameter must use { one_of = [...] }",
      ),
      (
        "fractal = \"julia\"\njulia = { c = { one_of = [[0.1]] } }",
        "each one_of entry must be [real, imaginary]",
      ),
      (
        "fractal = \"julia\"\njulia = { c = { one_of = [[0.0, 0.0]], extra = 1 } }",
        "unexpected key \"extra\"",
      ),
    ] {
      let error = parse(config).unwrap_err();
      assert!(error.contains(message), "for {config} got: {error}");
    }
  }

  #[test]
  fn unknown_fractal_name_is_rejected() {
    assert!(parse("fractal = \"newton\"\n").is_err());
  }

  #[test]
  fn a_two_value_bare_list_is_a_range_not_a_set() {
    let art: ArtConfig =
      toml::from_str("fractal = \"multibrot\"\nmultibrot = { power = [3, 5] }").unwrap();
    match art.multibrot.unwrap().power.unwrap() {
      Param::Range([3, 5]) => {}
      other => panic!("expected a range, got {other:?}"),
    }

    let art: ArtConfig =
      toml::from_str("fractal = \"multibrot\"\nmultibrot = { power = { one_of = [3, 5] } }")
        .unwrap();
    match art.multibrot.unwrap().power.unwrap() {
      Param::OneOf { one_of } if one_of == [3, 5] => {}
      other => panic!("expected a finite set, got {other:?}"),
    }
  }

  #[test]
  fn rejects_domains_with_no_points() {
    for config in [
      "fractal = \"julia\"\njulia = { c = [[[0.0, 0.0], [1.0, 1.0]], \"intersection\", [[2.0, 0.0], [3.0, 1.0]]] }",
      "fractal = \"multibrot\"\nmultibrot = { power = [[2, 3], \"intersection\", [6, 8]] }",
    ] {
      let error = parse(config).unwrap_err();
      assert!(error.contains("domain is empty"), "for {config} got: {error}");
    }
  }
}
