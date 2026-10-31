mod geometric;

use crate::config::{ArtConfig, FractalType, JuliaParameter, PowerParameter};
use rand::{rngs::StdRng, RngExt, SeedableRng};
use std::f64::consts::TAU;

const DEFAULT_MAX_ITERATIONS: usize = 80;
const DEFAULT_MULTIBROT_POWER: u8 = 3;

pub fn generate_ascii(
  fractal: FractalType,
  width: usize,
  height: usize,
  options: &ArtConfig,
) -> Vec<String> {
  if matches!(fractal, FractalType::None) {
    return Vec::new();
  }
  let mut rng = match options.seed {
    Some(seed) => StdRng::seed_from_u64(seed),
    None => StdRng::from_rng(&mut rand::rng()),
  };
  let fractal = if matches!(fractal, FractalType::Random) {
    const CHOICES: [FractalType; 10] = [
      FractalType::MandelbrotSet,
      FractalType::JuliaSet,
      FractalType::BurningShip,
      FractalType::Tricorn,
      FractalType::Multibrot,
      FractalType::SierpinskiTriangle,
      FractalType::SierpinskiCarpet,
      FractalType::CantorDust,
      FractalType::Vicsek,
      FractalType::KochCurve,
    ];
    CHOICES[rng.random_range(0..CHOICES.len())]
  } else {
    fractal
  };
  if matches!(
    fractal,
    FractalType::SierpinskiTriangle
      | FractalType::SierpinskiCarpet
      | FractalType::CantorDust
      | FractalType::Vicsek
      | FractalType::KochCurve
  ) {
    return geometric::render(fractal, width, height, options.depth.unwrap_or(3));
  }
  let julia_c = if matches!(fractal, FractalType::JuliaSet) {
    match options.julia_c {
      Some(JuliaParameter::Fixed([real, imaginary])) => (real, imaginary),
      Some(JuliaParameter::Random(_)) | None => {
        let theta: f64 = rng.random_range(0.0..TAU);
        (
          0.5 * theta.cos() - 0.25 * (2.0 * theta).cos(),
          0.5 * theta.sin() - 0.25 * (2.0 * theta).sin(),
        )
      }
    }
  } else {
    (0.0, 0.0)
  };
  let multibrot_power = match options.multibrot_power {
    Some(PowerParameter::Fixed(power)) => power,
    Some(PowerParameter::Random(_)) if matches!(fractal, FractalType::Multibrot) => {
      rng.random_range(2..=8)
    }
    _ => DEFAULT_MULTIBROT_POWER,
  };
  let max_iterations = options.max_iterations.unwrap_or(DEFAULT_MAX_ITERATIONS);
  let escape_radius = if matches!(fractal, FractalType::JuliaSet) {
    julia_c.0.hypot(julia_c.1).max(2.0)
  } else {
    2.0
  };
  let (xmin, xmax, ymin, ymax) = match fractal {
    FractalType::JuliaSet => (-1.5, 1.5, -1.0, 1.0),
    FractalType::BurningShip => (-2.2, 1.2, -2.0, 0.5),
    _ => (-2.0, 1.0, -1.2, 1.2),
  };
  (0..height)
    .map(|row| {
      let cy = ymin + (ymax - ymin) * (row as f64 + 0.5) / height as f64;
      (0..width)
        .map(|column| {
          let cx = xmin + (xmax - xmin) * (column as f64 + 0.5) / width as f64;
          let (mut x, mut y, a, b) = if matches!(fractal, FractalType::JuliaSet) {
            (cx, cy, julia_c.0, julia_c.1)
          } else {
            (0.0, 0.0, cx, cy)
          };
          let mut iterations = 0;
          while x.hypot(y) <= escape_radius && iterations < max_iterations {
            let (next_x, next_y) = match fractal {
              FractalType::BurningShip => (x * x - y * y + a, 2.0 * (x * y).abs() + b),
              FractalType::Tricorn => (x * x - y * y + a, -2.0 * x * y + b),
              FractalType::Multibrot => {
                let (mut real, mut imaginary) = (1.0, 0.0);
                for _ in 0..multibrot_power {
                  (real, imaginary) = (real * x - imaginary * y, real * y + imaginary * x);
                }
                (real + a, imaginary + b)
              }
              _ => (x * x - y * y + a, 2.0 * x * y + b),
            };
            x = next_x;
            y = next_y;
            iterations += 1;
          }
          match iterations {
            n if n == max_iterations && x.hypot(y) <= escape_radius => '█',
            n if n > max_iterations / 2 => '▓',
            n if n > max_iterations / 4 => '▒',
            n if n > max_iterations / 8 => '░',
            _ => ' ',
          }
        })
        .collect()
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  fn options(config: &str) -> ArtConfig {
    toml::from_str(config).unwrap()
  }

  #[test]
  fn all_fractals_have_requested_dimensions() {
    for fractal in [
      FractalType::MandelbrotSet,
      FractalType::JuliaSet,
      FractalType::BurningShip,
      FractalType::Tricorn,
      FractalType::Multibrot,
      FractalType::SierpinskiTriangle,
      FractalType::SierpinskiCarpet,
      FractalType::CantorDust,
      FractalType::Vicsek,
      FractalType::KochCurve,
    ] {
      let rows = generate_ascii(fractal, 30, 10, &options("seed = 42"));
      assert_eq!(rows.len(), 10);
      assert!(rows.iter().all(|row| row.chars().count() == 30));
      assert!(rows.iter().any(|row| row.contains('█')));
    }
  }
  #[test]
  fn seeded_julia_is_reproducible() {
    assert_eq!(
      generate_ascii(FractalType::JuliaSet, 40, 20, &options("seed = 123")),
      generate_ascii(FractalType::JuliaSet, 40, 20, &options("seed = 123"))
    );
    assert!(generate_ascii(FractalType::None, 40, 20, &options("")).is_empty());
    assert_eq!(
      generate_ascii(FractalType::Random, 40, 20, &options("seed = 123")),
      generate_ascii(FractalType::Random, 40, 20, &options("seed = 123"))
    );
  }
  #[test]
  fn fixed_julia_parameters_are_reproducible_and_change_the_image() {
    let first = options("julia_c = [-0.8, 0.156]\nseed = 1");
    let same = options("julia_c = [-0.8, 0.156]\nseed = 2");
    let different = options("julia_c = [0.0, 0.0]");
    let render = |options: &ArtConfig| generate_ascii(FractalType::JuliaSet, 80, 40, options);
    assert_eq!(render(&first), render(&same));
    assert_ne!(render(&first), render(&different));
  }

  #[test]
  fn escaping_on_final_iteration_is_not_bounded() {
    let image = generate_ascii(
      FractalType::MandelbrotSet,
      10,
      10,
      &options("max_iterations = 1"),
    );
    assert_ne!(image[0].chars().next(), Some('█'));
    assert_eq!(image[5].chars().nth(5), Some('█'));
  }

  #[test]
  fn iteration_limit_changes_escape_time_image() {
    let low = options("max_iterations = 4");
    let high = options("max_iterations = 80");
    assert_ne!(
      generate_ascii(FractalType::MandelbrotSet, 80, 40, &low),
      generate_ascii(FractalType::MandelbrotSet, 80, 40, &high)
    );
  }

  #[test]
  fn multibrot_power_and_random_parameters_affect_rendering() {
    let square = options("multibrot_power = 2");
    let cube = options("multibrot_power = 3");
    assert_eq!(
      generate_ascii(FractalType::Multibrot, 80, 40, &square),
      generate_ascii(FractalType::MandelbrotSet, 80, 40, &square)
    );
    assert_ne!(
      generate_ascii(FractalType::Multibrot, 80, 40, &square),
      generate_ascii(FractalType::Multibrot, 80, 40, &cube)
    );
    let random = options("multibrot_power = \"random\"\nseed = 123");
    assert_eq!(
      generate_ascii(FractalType::Multibrot, 80, 40, &random),
      generate_ascii(FractalType::Multibrot, 80, 40, &random)
    );
    let random_julia = options("julia_c = \"random\"\nseed = 123");
    let implicit_julia = options("seed = 123");
    assert_eq!(
      generate_ascii(FractalType::JuliaSet, 80, 40, &random_julia),
      generate_ascii(FractalType::JuliaSet, 80, 40, &implicit_julia)
    );
  }
}
