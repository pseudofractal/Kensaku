use crate::config::FractalType;

/// Render finite iterations of self-similar sets on a character grid.
pub fn render(fractal: FractalType, width: usize, height: usize, depth: u8) -> Vec<String> {
  if matches!(fractal, FractalType::KochCurve) {
    return koch_curve(width, height, depth);
  }
  (0..height)
    .map(|row| {
      (0..width)
        .map(|column| {
          const SAMPLES_PER_AXIS: usize = 4;
          let filled = (0..SAMPLES_PER_AXIS).any(|sample_y| {
            let y =
              (row as f64 + (sample_y as f64 + 0.5) / SAMPLES_PER_AXIS as f64) / height as f64;
            (0..SAMPLES_PER_AXIS).any(|sample_x| {
              let x =
                (column as f64 + (sample_x as f64 + 0.5) / SAMPLES_PER_AXIS as f64) / width as f64;
              match fractal {
                FractalType::SierpinskiTriangle => triangle_contains(x, y, depth),
                _ => square_contains(fractal, x, y, depth),
              }
            })
          });
          if filled {
            '█'
          } else {
            ' '
          }
        })
        .collect()
    })
    .collect()
}

fn triangle_contains(x: f64, y: f64, depth: u8) -> bool {
  let mut coordinates = [1.0 - y, x - 0.5 + y / 2.0, 0.5 + y / 2.0 - x];
  if coordinates.iter().any(|coordinate| *coordinate < 0.0) {
    return false;
  }
  for _ in 0..depth {
    let Some(corner) = coordinates.iter().position(|coordinate| *coordinate >= 0.5) else {
      return false;
    };
    for coordinate in &mut coordinates {
      *coordinate *= 2.0;
    }
    coordinates[corner] -= 1.0;
  }
  true
}

fn square_contains(fractal: FractalType, mut x: f64, mut y: f64, depth: u8) -> bool {
  for _ in 0..depth {
    x *= 3.0;
    y *= 3.0;
    let column = x.floor() as u8;
    let row = y.floor() as u8;
    let keep = match fractal {
      FractalType::SierpinskiCarpet => column != 1 || row != 1,
      FractalType::CantorDust => column != 1 && row != 1,
      FractalType::Vicsek => column == 1 || row == 1,
      _ => unreachable!("only square fractals are passed here"),
    };
    if !keep {
      return false;
    }
    x -= f64::from(column);
    y -= f64::from(row);
  }
  true
}

type Point = (f64, f64);

fn koch_segments(start: Point, end: Point, depth: u8, segments: &mut Vec<(Point, Point)>) {
  if depth == 0 {
    segments.push((start, end));
    return;
  }
  let dx = (end.0 - start.0) / 3.0;
  let dy = (end.1 - start.1) / 3.0;
  let first = (start.0 + dx, start.1 + dy);
  let second = (start.0 + 2.0 * dx, start.1 + 2.0 * dy);
  let peak = (
    first.0 + 0.5 * dx + 3.0_f64.sqrt() / 2.0 * dy,
    first.1 + 0.5 * dy - 3.0_f64.sqrt() / 2.0 * dx,
  );
  for (a, b) in [(start, first), (first, peak), (peak, second), (second, end)] {
    koch_segments(a, b, depth - 1, segments);
  }
}

fn koch_curve(width: usize, height: usize, depth: u8) -> Vec<String> {
  let mut grid = vec![vec![' '; width]; height];
  let mut segments = Vec::new();
  koch_segments((0.0, 0.85), (1.0, 0.85), depth, &mut segments);
  for (start, end) in segments {
    let x0 = start.0 * (width.saturating_sub(1)) as f64;
    let y0 = start.1 * (height.saturating_sub(1)) as f64;
    let x1 = end.0 * (width.saturating_sub(1)) as f64;
    let y1 = end.1 * (height.saturating_sub(1)) as f64;
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).ceil() as usize;
    for step in 0..=steps {
      let t = if steps == 0 {
        0.0
      } else {
        step as f64 / steps as f64
      };
      let x = (x0 + (x1 - x0) * t).round() as usize;
      let y = (y0 + (y1 - y0) * t).round() as usize;
      if let Some(cell) = grid.get_mut(y).and_then(|row| row.get_mut(x)) {
        *cell = '█';
      }
    }
  }
  grid
    .into_iter()
    .map(|row| row.into_iter().collect())
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn square_first_iteration_has_expected_copies() {
    for (fractal, count) in [
      (FractalType::SierpinskiCarpet, 8),
      (FractalType::CantorDust, 4),
      (FractalType::Vicsek, 5),
    ] {
      let rows = render(fractal, 3, 3, 1);
      assert_eq!(
        rows
          .iter()
          .flat_map(|row| row.chars())
          .filter(|c| *c == '█')
          .count(),
        count
      );
    }
  }

  #[test]
  fn cantor_dust_remains_visible_at_terminal_resolution() {
    let rows = render(FractalType::CantorDust, 40, 16, 3);
    assert!(rows.iter().filter(|row| row.contains('█')).count() >= 6);
    assert!(rows[7].chars().all(|character| character == ' '));
  }

  #[test]
  fn triangle_excludes_central_hole() {
    assert!(triangle_contains(0.5, 0.1, 1));
    assert!(!triangle_contains(0.5, 0.6, 1));
    assert!(!triangle_contains(0.0, 0.0, 1));
  }

  #[test]
  fn koch_segment_count_and_endpoints() {
    let mut segments = Vec::new();
    koch_segments((0.0, 0.0), (1.0, 0.0), 3, &mut segments);
    assert_eq!(segments.len(), 4_usize.pow(3));
    assert_eq!(segments.first().unwrap().0, (0.0, 0.0));
    assert_eq!(segments.last().unwrap().1, (1.0, 0.0));
  }
}
