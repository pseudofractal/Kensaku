mod config;

use config::Config;

fn main() {
    let config = Config::load();
    println!("Loaded Config: {:#?}", config);
}
