mod config;

use clap::Parser;

use config::{
    cfg_utils::{CONFIG, reload_config, watch_config},
    models::CmdArgs,
};

#[tokio::main]
async fn main() {
    println!("Certus Gateway Running");

    let args = CmdArgs::try_parse().unwrap();

    let config_path = args
        .config
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("certus.config.yaml");

    let _watcher = watch_config(config_path).unwrap();

    let initial = reload_config(config_path).unwrap();
    {
        let mut cfg = CONFIG.write();
        *cfg = initial;
    }

    println!("Config watcher started. Press Ctrl+C to exit.");
    println!(
        "Running on http://{}:{}",
        CONFIG.read().server.host,
        CONFIG.read().server.port
    );

    // Keep the program running until Ctrl+C
    tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");

    println!("\nShutting down...");
}
