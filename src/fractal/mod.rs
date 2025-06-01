pub mod mandelbrot;
pub mod julia;

use crate::config::FractalType;

pub fn generate_ascii(fractal: FractalType, width: usize, height: usize) -> Vec<String> {
    match fractal {
        FractalType::MandelbrotSet => mandelbrot::render(width, height),
        FractalType::JuliaSet => julia::render(width, height),
        FractalType::None => Vec::new(),
    }
}
