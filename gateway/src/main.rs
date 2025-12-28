mod config;
mod server;
mod logging;

use std::sync::Arc;

use axum::{Router, routing::any};
use clap::Parser;

use tokio::sync::mpsc;
use tracing::Level;
use tracing_subscriber::{FmtSubscriber, EnvFilter};

use crate::logging::log_util::LogChannelWriter;
use crate::server::{app_state::AppState, routing::routes};
use crate::{
    config::{
        cfg_utils::{reload_config, watch_config},
        models::CmdArgs,
    },
    server::app_state,
};

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(1024);

    let log_writer = LogChannelWriter { sender: tx };

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .json()
        .with_writer(log_writer)
        .finish();
        
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    tokio::spawn(async move {
        while let Some(log_json) = rx.recv().await {
            print!("{}", log_json); 
            
            // TODO: SAVE TO DB
        }
    });


    println!("Certus Gateway Running");

    let args = CmdArgs::try_parse().unwrap();

    let config_path = args
        .config
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("certus.config.yaml");

    let state =
        Arc::new(AppState::new(reload_config(config_path).await.unwrap()));
    let _watcher = match watch_config(config_path, state.clone()) {
        Ok(watcher) => Some(watcher),
        Err(_) => None,
    };

    routes::build_tree(state.clone());
    app_state::init_server_state(state.clone());

    let config = state.config.load();
    let port = config.server.port;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind TCP listener");

    println!("Config watcher started. Press Ctrl+C to exit.");
    println!("Running on port {}", port);

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
