mod config;
mod output;
mod info;

use config::Config;

fn main() {
    let config = Config::load();
    output::print_info(&config);
}
