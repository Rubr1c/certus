mod config;
mod db;
mod logging;
mod server;

use std::sync::Arc;

use axum::{Router, routing::any};
use clap::Parser;

use tokio::sync::{Mutex, mpsc};
use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::logging::log_util::LogChannelWriter;
use crate::server::{app_state::AppState, routing::routes};
use crate::{
    config::{
        cfg_utils::{reload_config, watch_config},
        models::CmdArgs,
    },
    db::db_utils,
    server::app_state,
};

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(1024);

    let log_writer = LogChannelWriter { sender: tx };

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive(Level::INFO.into()),
        )
        .with_writer(log_writer)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let args = CmdArgs::try_parse().unwrap();

    let config_path = args
        .config
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("certus.config.yaml");

    let conn = match db_utils::connect_db() {
        Ok(c) => c,
        Err(_) => panic!("Failed to connect to db"),
    };

    match db_utils::migrate(&conn) {
        Ok(_) => (),
        Err(_) => eprintln!("Error migrating db"),
    }

    let conn = Arc::new(Mutex::new(conn));
    let conn_clone = conn.clone();

    tokio::spawn(async move {
        while let Some(log_bytes) = rx.recv().await {
            let log_string = String::from_utf8_lossy(&log_bytes);

            print!("{}", log_string);
            let conn_guard = conn_clone.lock().await;
            match db_utils::save_log(&conn_guard, log_string.to_string()) {
                Ok(_) => (),
                Err(e) => eprint!("{}", e),
            }
        }
    });

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

    tracing::info!("Certus Gateway Running on port {}", port);
    println!("Config watcher started. Press Ctrl+C to exit.");

    let app =
        Router::new().route("/{*any}", any(routes::reroute)).with_state(state);

    let shutdown_signal = async {
        tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        tracing::info!("\nShutting down...");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .unwrap();
}
