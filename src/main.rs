mod config;
mod output;
mod info;
mod fractal;

use config::Config;

fn main() {
    let config = Config::load();
    output::print_info(&config);
}
