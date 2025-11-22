mod config;

use config::cfg_utils;

fn main() {
    println!("Certus Gateway Running");

    let config = match cfg_utils::read_config() {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Error loading config: {}", err);
            std::process::exit(1);
        }
    };

    println!("port: {}", config.port)
}
