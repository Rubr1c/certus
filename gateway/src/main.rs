mod config;
mod server;

use std::sync::Arc;

use axum::{routing::any, Router};
use clap::Parser;
use parking_lot::RwLock;

use crate::{config::{
    cfg_utils::{reload_config, watch_config},
    models::CmdArgs,
}, server::app_state};
use crate::server::{app_state::AppState, routing::routes};

#[tokio::main]
async fn main() {
    println!("Certus Gateway Running");

    let args = CmdArgs::try_parse().unwrap();

    let config_path = args
        .config
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("certus.config.yaml");


    let state = Arc::new(RwLock::new(AppState::new(reload_config(config_path).await.unwrap())));
    let _watcher = match watch_config(config_path, state.clone()) {
        Ok(watcher) => Some(watcher),
        Err(_) => None,
    };


    routes::build_tree(state.clone());
    app_state::init_server_state(state.clone());


    let listener = tokio::net::TcpListener::bind(format!(
        "0.0.0.0:{}",
        8080
    ))
    .await
    .expect("Failed to bind TCP listener");

    println!("Config watcher started. Press Ctrl+C to exit.");
    println!("Running on port {}", state.read().config.read().server.port);

    let app =
        Router::new().route("/{*any}", any(routes::reroute)).with_state(state);

    let shutdown_signal = async {
        tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        println!("\nShutting down...");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .unwrap();
}
