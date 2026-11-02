mod geometric;

use crate::config::{ArtConfig, FractalName, JuliaConfig, Param, MULTIBROT_POWER_RANGE};
use crate::domain::{self, Complex};
use rand::{rngs::StdRng, RngExt, SeedableRng};
use std::f64::consts::TAU;

const DEFAULT_MAX_ITERATIONS: usize = 80;
const DEFAULT_MULTIBROT_POWER: u8 = 3;

/// Render the fractal selected by `options` on a `width` × `height` grid.
///
/// `options` is expected to have passed `ArtConfig::validate`. `None` and
/// `FractalName::None` produce no art. An empty parameter domain, which
/// validation rejects, falls back to the parameter's standard random domain so
/// direct callers still get an image.
pub fn generate_ascii(width: usize, height: usize, options: &ArtConfig) -> Vec<String> {
  let fractal = options.fractal.unwrap_or(FractalName::None);
  if matches!(fractal, FractalName::None) {
    return Vec::new();
  }
  let mut rng = match options.seed {
    Some(seed) => StdRng::seed_from_u64(seed),
    None => StdRng::from_rng(&mut rand::rng()),
  };
  let fractal = if matches!(fractal, FractalName::Random) {
    const CHOICES: [FractalName; 10] = [
      FractalName::Mandelbrot,
      FractalName::Julia,
      FractalName::BurningShip,
      FractalName::Tricorn,
      FractalName::Multibrot,
      FractalName::SierpinskiTriangle,
      FractalName::SierpinskiCarpet,
      FractalName::CantorDust,
      FractalName::Vicsek,
      FractalName::KochCurve,
    ];
    CHOICES[rng.random_range(0..CHOICES.len())]
  } else {
    fractal
  };
  if matches!(
    fractal,
    FractalName::SierpinskiTriangle
      | FractalName::SierpinskiCarpet
      | FractalName::CantorDust
      | FractalName::Vicsek
      | FractalName::KochCurve
  ) {
    return geometric::render(fractal, width, height, options.depth.unwrap_or(3));
  }
  let julia_c = if matches!(fractal, FractalName::Julia) {
    sample_julia(options.julia.as_ref(), &mut rng)
  } else {
    [0.0, 0.0]
  };
  let multibrot_power = if matches!(fractal, FractalName::Multibrot) {
    match &options.multibrot {
      Some(config) => config
        .power
        .as_ref()
        .map(|param| sample_power(param, &mut rng))
        .unwrap_or(DEFAULT_MULTIBROT_POWER),
      None => DEFAULT_MULTIBROT_POWER,
    }
  } else {
    DEFAULT_MULTIBROT_POWER
  };
  let max_iterations = options.max_iterations.unwrap_or(DEFAULT_MAX_ITERATIONS);
  let escape_radius = if matches!(fractal, FractalName::Julia) {
    julia_c[0].hypot(julia_c[1]).max(2.0)
  } else {
    2.0
  };
  let (xmin, xmax, ymin, ymax) = match fractal {
    FractalName::Julia => (-1.5, 1.5, -1.0, 1.0),
    FractalName::BurningShip => (-2.2, 1.2, -2.0, 0.5),
    _ => (-2.0, 1.0, -1.2, 1.2),
  };
  (0..height)
    .map(|row| {
      let cy = ymin + (ymax - ymin) * (row as f64 + 0.5) / height as f64;
      (0..width)
        .map(|column| {
          let cx = xmin + (xmax - xmin) * (column as f64 + 0.5) / width as f64;
          let (mut x, mut y, a, b) = if matches!(fractal, FractalName::Julia) {
            (cx, cy, julia_c[0], julia_c[1])
          } else {
            (0.0, 0.0, cx, cy)
          };
          let mut iterations = 0;
          while x.hypot(y) <= escape_radius && iterations < max_iterations {
            let (next_x, next_y) = match fractal {
              FractalName::BurningShip => (x * x - y * y + a, 2.0 * (x * y).abs() + b),
              FractalName::Tricorn => (x * x - y * y + a, -2.0 * x * y + b),
              FractalName::Multibrot => {
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

/// Resolve the Julia constant `c`, defaulting to a random point on the main
/// cardioid boundary when `[art.julia]` is absent or leaves `c` unset.
fn sample_julia(config: Option<&JuliaConfig>, rng: &mut StdRng) -> Complex {
  match config.and_then(|julia| julia.c.as_ref()) {
    None => cardioid_point(rng),
    Some(param) => sample_complex(param, rng),
  }
}

fn sample_complex(param: &Param<Complex>, rng: &mut StdRng) -> Complex {
  match param {
    Param::Fixed(value) => *value,
    Param::Random => cardioid_point(rng),
    Param::Range([from, to]) => domain::sample_complex_range(*from, *to, rng),
    Param::Fold(tokens) => {
      // Validation rejects empty domains; the fallback keeps direct callers total.
      domain::sample_complex_domain(tokens, rng).unwrap_or_else(|| cardioid_point(rng))
    }
    Param::OneOf { one_of } => pick(one_of, rng),
  }
}

fn sample_power(param: &Param<u8>, rng: &mut StdRng) -> u8 {
  match param {
    Param::Fixed(value) => *value,
    Param::Random => rng.random_range(MULTIBROT_POWER_RANGE),
    Param::Range([from, to]) => rng.random_range(*from..=*to),
    Param::Fold(tokens) => {
      // Validation rejects empty domains; the fallback keeps direct callers total.
      domain::sample_power_domain(tokens, rng)
        .unwrap_or_else(|| rng.random_range(MULTIBROT_POWER_RANGE))
    }
    Param::OneOf { one_of } => pick(one_of, rng),
  }
}

fn pick<T: Copy>(choices: &[T], rng: &mut StdRng) -> T {
  choices[rng.random_range(0..choices.len())]
}

/// A random point on the boundary of the Mandelbrot set's main cardioid,
/// uniform in the cardioid parameter θ.
fn cardioid_point(rng: &mut StdRng) -> Complex {
  let theta: f64 = rng.random_range(0.0..TAU);
  [
    0.5 * theta.cos() - 0.25 * (2.0 * theta).cos(),
    0.5 * theta.sin() - 0.25 * (2.0 * theta).sin(),
  ]
}

#[cfg(test)]
mod tests {
  use super::*;

  fn options(fractal: &str, config: &str) -> ArtConfig {
    toml::from_str(&format!("fractal = \"{fractal}\"\n{config}")).unwrap()
  }

  #[test]
  fn all_fractals_have_requested_dimensions() {
    for fractal in [
      "mandelbrot",
      "julia",
      "burning_ship",
      "tricorn",
      "multibrot",
      "sierpinski_triangle",
      "sierpinski_carpet",
      "cantor_dust",
      "vicsek",
      "koch_curve",
    ] {
      let rows = generate_ascii(30, 10, &options(fractal, "seed = 42"));
      assert_eq!(rows.len(), 10);
      assert!(rows.iter().all(|row| row.chars().count() == 30));
      assert!(rows.iter().any(|row| row.contains('█')));
    }
  }

  #[test]
  fn seeded_julia_is_reproducible() {
    assert_eq!(
      generate_ascii(40, 20, &options("julia", "seed = 123")),
      generate_ascii(40, 20, &options("julia", "seed = 123"))
    );
    assert!(generate_ascii(40, 20, &options("none", "")).is_empty());
    assert_eq!(
      generate_ascii(40, 20, &options("random", "seed = 123")),
      generate_ascii(40, 20, &options("random", "seed = 123"))
    );
  }

  #[test]
  fn fixed_julia_parameters_are_reproducible_and_change_the_image() {
    let first = options("julia", "julia = { c = [-0.8, 0.156] }\nseed = 1");
    let same = options("julia", "julia = { c = [-0.8, 0.156] }\nseed = 2");
    let different = options("julia", "julia = { c = [0.0, 0.0] }");
    let render = |options: &ArtConfig| generate_ascii(80, 40, options);
    assert_eq!(render(&first), render(&same));
    assert_ne!(render(&first), render(&different));
  }

  #[test]
  fn julia_domain_modes_render_and_differ() {
    let fixed = generate_ascii(60, 30, &options("julia", "julia = { c = [-0.8, 0.156] }"));
    let rect = generate_ascii(
      60,
      30,
      &options("julia", "julia = { c = [[-1.0, -1.0], [1.0, 1.0]] }"),
    );
    let fold = generate_ascii(
      60,
      30,
      &options(
        "julia",
        "seed = 1\njulia = { c = [[[-1.0, -1.0], [1.0, 1.0]], \"difference\", [[-0.25, -0.25], [0.25, 0.25]]] }",
      ),
    );
    for rows in [&fixed, &rect, &fold] {
      assert_eq!(rows.len(), 30);
      assert!(rows.iter().all(|row| row.chars().count() == 60));
    }
    assert_ne!(fixed, rect);
  }

  #[test]
  fn multibrot_domain_modes_render() {
    for config in [
      "multibrot = { power = [[2, 3], \"union\", [6, 8]] }",
      "multibrot = { power = { one_of = [3, 5, 7] } }",
      "multibrot = { power = [2, 6] }",
    ] {
      let rows = generate_ascii(40, 20, &options("multibrot", config));
      assert_eq!(rows.len(), 20);
      assert!(rows.iter().all(|row| row.chars().count() == 40));
    }
  }

  #[test]
  fn escaping_on_final_iteration_is_not_bounded() {
    let image = generate_ascii(10, 10, &options("mandelbrot", "max_iterations = 1"));
    assert_ne!(image[0].chars().next(), Some('█'));
    assert_eq!(image[5].chars().nth(5), Some('█'));
  }

  #[test]
  fn iteration_limit_changes_escape_time_image() {
    let low = options("mandelbrot", "max_iterations = 4");
    let high = options("mandelbrot", "max_iterations = 80");
    assert_ne!(generate_ascii(80, 40, &low), generate_ascii(80, 40, &high));
  }

  #[test]
  fn multibrot_power_and_random_parameters_affect_rendering() {
    let square = options("multibrot", "multibrot = { power = 2 }");
    let cube = options("multibrot", "multibrot = { power = 3 }");
    let mandelbrot = options("mandelbrot", "");
    assert_eq!(
      generate_ascii(80, 40, &square),
      generate_ascii(80, 40, &mandelbrot)
    );
    assert_ne!(
      generate_ascii(80, 40, &square),
      generate_ascii(80, 40, &cube)
    );
    let random = options(
      "multibrot",
      "multibrot = { power = \"random\" }\nseed = 123",
    );
    assert_eq!(
      generate_ascii(80, 40, &random),
      generate_ascii(80, 40, &random)
    );
    let random_julia = options("julia", "julia = { c = \"random\" }\nseed = 123");
    let implicit_julia = options("julia", "seed = 123");
    assert_eq!(
      generate_ascii(80, 40, &random_julia),
      generate_ascii(80, 40, &implicit_julia)
    );
  }
}
