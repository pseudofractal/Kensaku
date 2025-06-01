use rand::{self, Rng};
use std::f64::consts::TAU;

fn random_init() -> (f64, f64) {
    let mut rng = rand::rng();
    loop {
        // Pick a Point in the Main Carotid for Better Results
        let theta = rng.random_range(0.0..TAU);
        let cx = 0.5 * theta.cos() - 0.25 * (2.0 * theta).cos();
        let cy = 0.5 * theta.sin() - 0.25 * (2.0 * theta).sin();
        // keep only those with sufficiently large |Im|
        if cy.abs() > 0.5 {
            return (cx, cy);
        }
    }
}

pub fn render(width: usize, height: usize) -> Vec<String> {
    let (cx, cy) = random_init();
    let max_iter = 80;
    let x_min = -1.5;
    let x_max = 1.5;
    let y_min = -1.0;
    let y_max = 1.0;
    let mut rows = Vec::with_capacity(height);
    for j in 0..height {
        let mut line = String::with_capacity(width);
        let y0 = y_min + (y_max - y_min) * (j as f64 / (height as f64));
        for i in 0..width {
            let x0 = x_min + (x_max - x_min) * (i as f64 / (width as f64));
            let mut x = x0;
            let mut y = y0;
            let mut iter = 0;
            while x * x + y * y <= 4.0 && iter < max_iter {
                let xt = x * x - y * y + cx;
                y = 2.0 * x * y + cy;
                x = xt;
                iter += 1;
            }
            let c = match iter {
                n if n == max_iter => '█',
                n if n > max_iter / 2 => '▓',
                n if n > max_iter / 4 => '▒',
                n if n > max_iter / 8 => '░',
                _ => ' ',
            };

            line.push(c);
        }
        rows.push(line);
    }
    rows
}

