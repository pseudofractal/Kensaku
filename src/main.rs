mod config;
mod domain;
mod fractal;
mod info;
mod output;

use config::Config;

fn main() {
  let arguments: Vec<_> = std::env::args_os().skip(1).collect();
  if !arguments.is_empty() {
    match arguments.as_slice() {
      [arg] if arg == "--help" || arg == "-h" => {
        println!("Kensaku {}\n\nUsage: kensaku [--help | --version | --list-fractals]\n\nRun without arguments to display system information.\nSee the README for configuration options.", env!("CARGO_PKG_VERSION"));
      }
      [arg] if arg == "--version" || arg == "-V" => println!("kensaku {}", env!("CARGO_PKG_VERSION")),
      [arg] if arg == "--list-fractals" => println!("random\nnone\nmandelbrot\njulia\nburning_ship\ntricorn\nmultibrot\nsierpinski_triangle\nsierpinski_carpet\ncantor_dust\nvicsek\nkoch_curve"),
      _ => {
        eprintln!("kensaku: unrecognized arguments; use --help for usage");
        std::process::exit(2);
      }
    }
    return;
  }
  match Config::load() {
    Ok(config) => output::print_info(&config),
    Err(error) => {
      eprintln!("kensaku: {error}");
      std::process::exit(1);
    }
  }
}
