//! Parameter-domain set semantics: which points a domain expression contains
//! and how to draw uniformly from it.
//!
//! A domain is a fold of axis-aligned ranges combined with set operations.
//! Every operation preserves the grid defined by the range edges, so
//! membership is constant on each grid cell. Sampling splits the domain into
//! those cells and samples uniformly by area, falling back to length for
//! line-only domains and a uniform choice for point-only domains.

use rand::{rngs::StdRng, RngExt};

/// A complex number written as a `[real, imaginary]` pair.
pub type Complex = [f64; 2];

/// One term of a domain expression: a range `[from, to]` or a set operation.
#[derive(Debug, Clone)]
pub enum Token<T> {
  Range([T; 2]),
  Op(SetOp),
}

/// How two ranges are combined in a domain expression.
#[derive(Debug, Clone, Copy)]
pub enum SetOp {
  Union,
  Intersection,
  Difference,
}

/// Evaluate an alternating `range op range ...` fold left-to-right against a
/// candidate point. A list with no ranges contains nothing.
fn contains<T: Copy>(tokens: &[Token<T>], point: T, inside: impl Fn(T, T, T) -> bool) -> bool {
  let Some(first) = tokens.first() else {
    return false;
  };
  let mut accumulated = match first {
    Token::Range([from, to]) => inside(*from, *to, point),
    Token::Op(_) => false, // unreachable for validated input
  };
  for pair in tokens[1..].chunks(2) {
    let [op_token, range_token] = pair else {
      continue; // unreachable for validated input
    };
    let op = match op_token {
      Token::Op(op) => *op,
      Token::Range(_) => continue, // unreachable for validated input
    };
    let (from, to) = match range_token {
      Token::Range([from, to]) => (*from, *to),
      Token::Op(_) => continue, // unreachable for validated input
    };
    let point_inside = inside(from, to, point);
    accumulated = match op {
      SetOp::Union => accumulated || point_inside,
      SetOp::Intersection => accumulated && point_inside,
      SetOp::Difference => accumulated && !point_inside,
    };
  }
  accumulated
}

/// A rectangular piece of a complex domain that lies entirely inside it.
struct ComplexCell {
  from: Complex,
  to: Complex,
}

impl ComplexCell {
  fn width(&self) -> f64 {
    self.to[0] - self.from[0]
  }

  fn height(&self) -> f64 {
    self.to[1] - self.from[1]
  }

  fn area(&self) -> f64 {
    self.width() * self.height()
  }

  /// Length of a one-dimensional cell, or zero for points and rectangles.
  fn length(&self) -> f64 {
    match (self.width(), self.height()) {
      (width, 0.0) if width > 0.0 => width,
      (0.0, height) if height > 0.0 => height,
      _ => 0.0,
    }
  }

  fn center(&self) -> Complex {
    [
      if self.width() == 0.0 {
        self.from[0]
      } else {
        (self.from[0] + self.to[0]) / 2.0
      },
      if self.height() == 0.0 {
        self.from[1]
      } else {
        (self.from[1] + self.to[1]) / 2.0
      },
    ]
  }
}

/// Split a complex domain along every range edge and keep the cells whose
/// center lies inside it. The grid includes the edges themselves, so a point
/// or line inside a larger fold is represented as a zero-area cell instead of
/// being dropped.
fn complex_cells(tokens: &[Token<Complex>]) -> Vec<ComplexCell> {
  let mut xs = Vec::new();
  let mut ys = Vec::new();
  for token in tokens {
    if let Token::Range([from, to]) = token {
      xs.extend([from[0], to[0]]);
      ys.extend([from[1], to[1]]);
    }
  }
  xs.sort_by(f64::total_cmp);
  xs.dedup();
  ys.sort_by(f64::total_cmp);
  ys.dedup();
  let mut cells = Vec::new();
  for x in intervals(&xs) {
    for y in intervals(&ys) {
      let cell = ComplexCell {
        from: [x.0, y.0],
        to: [x.1, y.1],
      };
      if contains(tokens, cell.center(), complex_inside) {
        cells.push(cell);
      }
    }
  }
  cells
}

/// Every point and open interval along one axis. The degenerate intervals keep
/// folded points and lines from disappearing when another range adds an
/// interior edge.
fn intervals(values: &[f64]) -> Vec<(f64, f64)> {
  let mut intervals = Vec::new();
  for (index, value) in values.iter().enumerate() {
    intervals.push((*value, *value));
    if let Some(next) = values.get(index + 1) {
      intervals.push((*value, *next));
    }
  }
  intervals
}

fn complex_inside(from: Complex, to: Complex, point: Complex) -> bool {
  point[0] >= from[0] && point[0] <= to[0] && point[1] >= from[1] && point[1] <= to[1]
}

fn sample_axis(low: f64, high: f64, rng: &mut StdRng) -> f64 {
  if low == high {
    low
  } else {
    low + (high - low) * rng.random_range(0.0..1.0)
  }
}

/// Draw a uniformly random point from a rectangular range.
pub fn sample_complex_range(from: Complex, to: Complex, rng: &mut StdRng) -> Complex {
  [
    sample_axis(from[0], to[0], rng),
    sample_axis(from[1], to[1], rng),
  ]
}

/// Draw a uniformly random point from a complex domain.
///
/// Cells with area are preferred, then cells with length, then points, so the
/// domain is sampled uniformly in its highest dimension. Returns `None` for an
/// empty domain, which validation rejects.
pub fn sample_complex_domain(tokens: &[Token<Complex>], rng: &mut StdRng) -> Option<Complex> {
  let cells = complex_cells(tokens);
  if let Some(cell) = pick_weighted(
    cells.iter().filter(|cell| cell.area() > 0.0),
    ComplexCell::area,
    rng,
  ) {
    return Some(sample_complex_range(cell.from, cell.to, rng));
  }
  if let Some(cell) = pick_weighted(
    cells.iter().filter(|cell| cell.length() > 0.0),
    ComplexCell::length,
    rng,
  ) {
    return Some(sample_complex_range(cell.from, cell.to, rng));
  }
  pick_weighted(cells.iter(), |_| 1.0, rng).map(ComplexCell::center)
}

/// Whether a complex domain contains no points.
pub fn complex_domain_is_empty(tokens: &[Token<Complex>]) -> bool {
  complex_cells(tokens).is_empty()
}

fn pick_weighted<'a, T>(
  items: impl Iterator<Item = &'a T>,
  weight: impl Fn(&T) -> f64,
  rng: &mut StdRng,
) -> Option<&'a T> {
  let candidates: Vec<(&T, f64)> = items.map(|item| (item, weight(item))).collect();
  let total: f64 = candidates.iter().map(|(_, weight)| weight).sum();
  if !total.is_finite() || total <= 0.0 {
    return None;
  }
  let mut draw = rng.random_range(0.0..total);
  for (item, weight) in &candidates {
    if draw < *weight {
      return Some(item);
    }
    draw -= *weight;
  }
  candidates.last().map(|(item, _)| *item)
}

/// An inclusive run of integers that lies entirely inside a power domain.
struct PowerCell {
  low: u8,
  high: u8,
}

impl PowerCell {
  fn count(&self) -> u16 {
    u16::from(self.high) - u16::from(self.low) + 1
  }
}

/// Split a power domain at every range edge; membership is constant on each
/// run of integers because the edges bound the ranges.
fn power_cells(tokens: &[Token<u8>]) -> Vec<PowerCell> {
  let mut bounds: Vec<u16> = Vec::new();
  for token in tokens {
    if let Token::Range([from, to]) = token {
      bounds.push(u16::from(*from));
      bounds.push(u16::from(*to) + 1);
    }
  }
  bounds.sort_unstable();
  bounds.dedup();
  let mut cells = Vec::new();
  for pair in bounds.windows(2) {
    // A boundary of 256 can only be the final value, so it never starts a cell.
    let low = u8::try_from(pair[0]).expect("interior boundaries fit in u8");
    let high = u8::try_from(pair[1] - 1).expect("interior boundaries fit in u8");
    if contains(tokens, low, power_inside) {
      cells.push(PowerCell { low, high });
    }
  }
  cells
}

fn power_inside(from: u8, to: u8, point: u8) -> bool {
  from <= point && point <= to
}

/// Draw a uniformly random integer from a power domain. Returns `None` for an
/// empty domain, which validation rejects.
pub fn sample_power_domain(tokens: &[Token<u8>], rng: &mut StdRng) -> Option<u8> {
  let cells = power_cells(tokens);
  let total: u32 = cells.iter().map(|cell| u32::from(cell.count())).sum();
  if total == 0 {
    return None;
  }
  let mut draw = rng.random_range(0..total);
  for cell in &cells {
    let count = u32::from(cell.count());
    if draw < count {
      return Some(cell.low + draw as u8);
    }
    draw -= count;
  }
  cells.last().map(|cell| cell.high)
}

/// Whether a power domain contains no values.
pub fn power_domain_is_empty(tokens: &[Token<u8>]) -> bool {
  power_cells(tokens).is_empty()
}

#[cfg(test)]
mod tests {
  use super::*;
  use rand::SeedableRng;

  fn rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
  }

  fn range(from: Complex, to: Complex) -> Token<Complex> {
    Token::Range([from, to])
  }

  #[test]
  fn an_empty_fold_contains_nothing_and_does_not_panic() {
    assert!(!contains::<Complex>(&[], [0.0, 0.0], complex_inside));
    assert!(complex_domain_is_empty(&[]));
    assert_eq!(sample_complex_domain(&[], &mut rng(1)), None);
    assert!(power_domain_is_empty(&[]));
    assert_eq!(sample_power_domain(&[], &mut rng(1)), None);
  }

  #[test]
  fn difference_hole_is_never_sampled() {
    let tokens = vec![
      range([-1.0, -1.0], [1.0, 1.0]),
      Token::Op(SetOp::Difference),
      range([-0.5, -0.5], [0.5, 0.5]),
    ];
    let mut rng = rng(7);
    for _ in 0..500 {
      let point = sample_complex_domain(&tokens, &mut rng).unwrap();
      let inside_hole = point[0] > -0.5 && point[0] < 0.5 && point[1] > -0.5 && point[1] < 0.5;
      assert!(!inside_hole, "sampled {point:?} inside the hole");
      assert!((-1.0..=1.0).contains(&point[0]) && (-1.0..=1.0).contains(&point[1]));
    }
  }

  #[test]
  fn union_samples_both_boxes() {
    let tokens = vec![
      range([-2.0, 0.0], [-1.0, 1.0]),
      Token::Op(SetOp::Union),
      range([1.0, 0.0], [2.0, 1.0]),
    ];
    let mut rng = rng(3);
    let (mut left, mut right) = (false, false);
    for _ in 0..200 {
      let point = sample_complex_domain(&tokens, &mut rng).unwrap();
      left |= point[0] < 0.0;
      right |= point[0] > 0.0;
    }
    assert!(left && right);
  }

  #[test]
  fn area_weighting_follows_the_union_areas() {
    // Areas 1 and 3, so roughly a quarter of the draws should land on the left.
    let tokens = vec![
      range([-2.0, 0.0], [-1.0, 1.0]),
      Token::Op(SetOp::Union),
      range([1.0, 0.0], [4.0, 1.0]),
    ];
    let mut rng = rng(21);
    let draws = 4000;
    let left = (0..draws)
      .filter(|_| sample_complex_domain(&tokens, &mut rng).unwrap()[0] < 0.0)
      .count();
    let fraction = left as f64 / draws as f64;
    assert!(
      (fraction - 0.25).abs() < 0.05,
      "left fraction was {fraction}"
    );
  }

  #[test]
  fn point_domain_samples_its_point() {
    let tokens = vec![range([0.5, -0.5], [0.5, -0.5])];
    assert_eq!(
      sample_complex_domain(&tokens, &mut rng(1)),
      Some([0.5, -0.5])
    );
  }

  #[test]
  fn union_of_points_samples_both_points() {
    let first = [0.1, 0.1];
    let second = [0.2, 0.2];
    let tokens = vec![
      range(first, first),
      Token::Op(SetOp::Union),
      range(second, second),
    ];
    assert!(!complex_domain_is_empty(&tokens));
    let mut rng = rng(5);
    let (mut first_seen, mut second_seen) = (false, false);
    for _ in 0..100 {
      let point = sample_complex_domain(&tokens, &mut rng).unwrap();
      assert!(point == first || point == second, "sampled {point:?}");
      first_seen |= point == first;
      second_seen |= point == second;
    }
    assert!(first_seen && second_seen);
  }

  #[test]
  fn union_of_lines_samples_both_lines() {
    let tokens = vec![
      range([0.0, 0.0], [0.0, 1.0]),
      Token::Op(SetOp::Union),
      range([1.0, 0.0], [1.0, 1.0]),
    ];
    assert!(!complex_domain_is_empty(&tokens));
    let mut rng = rng(13);
    let (mut left_seen, mut right_seen) = (false, false);
    for _ in 0..200 {
      let point = sample_complex_domain(&tokens, &mut rng).unwrap();
      assert!((0.0..=1.0).contains(&point[1]), "sampled {point:?}");
      if point[0] == 0.0 {
        left_seen = true;
      } else if point[0] == 1.0 {
        right_seen = true;
      } else {
        panic!("sampled off the lines at x = {}", point[0]);
      }
    }
    assert!(left_seen && right_seen);
  }

  #[test]
  fn crossing_lines_intersect_in_a_point() {
    let tokens = vec![
      range([-1.0, 0.0], [1.0, 0.0]),
      Token::Op(SetOp::Intersection),
      range([0.0, -1.0], [0.0, 1.0]),
    ];
    assert!(!complex_domain_is_empty(&tokens));
    assert_eq!(
      sample_complex_domain(&tokens, &mut rng(17)),
      Some([0.0, 0.0])
    );
  }

  #[test]
  fn touching_rectangles_intersect_in_a_line() {
    let tokens = vec![
      range([0.0, 0.0], [1.0, 1.0]),
      Token::Op(SetOp::Intersection),
      range([1.0, 0.0], [2.0, 1.0]),
    ];
    assert!(!complex_domain_is_empty(&tokens));
    let mut rng = rng(23);
    for _ in 0..20 {
      let point = sample_complex_domain(&tokens, &mut rng).unwrap();
      assert_eq!(point[0], 1.0);
      assert!((0.0..=1.0).contains(&point[1]), "sampled {point:?}");
    }
  }

  #[test]
  fn disjoint_intersection_is_empty() {
    let tokens = vec![
      range([0.0, 0.0], [1.0, 1.0]),
      Token::Op(SetOp::Intersection),
      range([2.0, 0.0], [3.0, 1.0]),
    ];
    assert!(complex_domain_is_empty(&tokens));
  }

  #[test]
  fn degenerate_range_samples_its_line() {
    let tokens = vec![range([-1.0, 0.5], [1.0, 0.5])];
    let mut rng = rng(9);
    for _ in 0..10 {
      let point = sample_complex_domain(&tokens, &mut rng).unwrap();
      assert_eq!(point[1], 0.5);
      assert!((-1.0..=1.0).contains(&point[0]));
    }
  }

  #[test]
  fn power_domains_respect_their_operations() {
    let union = vec![
      Token::Range([2u8, 3]),
      Token::Op(SetOp::Union),
      Token::Range([6, 8]),
    ];
    let mut rng = rng(11);
    let mut seen = [false; 9];
    for _ in 0..500 {
      let power = sample_power_domain(&union, &mut rng).unwrap();
      seen[usize::from(power)] = true;
    }
    assert_eq!(
      seen,
      [false, false, true, true, false, false, true, true, true]
    );

    let intersection = vec![
      Token::Range([2u8, 6]),
      Token::Op(SetOp::Intersection),
      Token::Range([4, 8]),
    ];
    for _ in 0..200 {
      let power = sample_power_domain(&intersection, &mut rng).unwrap();
      assert!((4..=6).contains(&power));
    }

    let disjoint = vec![
      Token::Range([2u8, 3]),
      Token::Op(SetOp::Intersection),
      Token::Range([6, 8]),
    ];
    assert!(power_domain_is_empty(&disjoint));
  }
}
