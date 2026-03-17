pub mod cli;
pub mod config;
pub mod connection;
pub mod controllers;
pub mod db;
pub mod error;
pub mod logging;
pub mod metrics;
pub mod middleware;
pub mod schema;
pub mod server;
pub mod tasks;
pub mod upstream;
pub mod websocket;

#[cfg(test)]
mod tests;

use std::{net::SocketAddr, sync::Arc};

use clap::Parser;
use tokio::sync::mpsc;

use crate::{
    logging::LogEntryDTO,
    metrics::MetricEvent,
    schema::ReqResSchemaDTO,
    server::state::{app_state::AppState, initializer},
};

pub async fn run() {
    let (log_tx, log_rx) = mpsc::channel::<LogEntryDTO>(1024);
    let (metrics_tx, metrics_rx) = mpsc::channel::<MetricEvent>(1024);
    let (schema_tx, schema_rx) = mpsc::channel::<ReqResSchemaDTO>(1024);

    server::bootstrap::tracing::run(log_tx);

    let args = Arc::new(cli::CmdArgs::parse());

    let config_path = args.config.as_str();

    let certus_routes = server::bootstrap::routes::run();
    let (certus_routes, log_broadcast_tx, metrics_broadcast_tx) =
        server::bootstrap::websocket::run(certus_routes, args.as_ref());

    let (log_conn, metrics_conn, schema_conn) = server::bootstrap::db::run();

    let config = config::parser::reload_config(config_path).await.unwrap();

    let tls = config.tls.clone();

    let state = Arc::new(
        AppState::new(
            config,
            metrics_conn.clone(),
            metrics_tx,
            schema_tx,
            log_broadcast_tx.clone(),
            metrics_broadcast_tx.clone(),
            args.clone(),
        )
        .await,
    );

    tasks::log_flusher::run(log_conn.clone(), log_rx, log_broadcast_tx).await;

    tasks::metric_flusher::run(
        metrics_conn.clone(),
        metrics_rx,
        metrics_broadcast_tx,
    )
    .await;

    tasks::schema_flusher::run(schema_conn, schema_rx).await;

    tasks::health_reaper::run(state.clone()).await;

    let _watcher =
        match config::watcher::watch_config(config_path, state.clone()).await {
            Ok(watcher) => Some(watcher),
            Err(_) => None,
        };

    initializer::init_server_state(state.clone()).await;

    let config = state.config.load();
    let port = config.server.port;

    tracing::info!("Certus Gateway Running on port {}", port);
    println!("Config watcher started. Press Ctrl+C to exit.");

    let app = server::bootstrap::router::run(certus_routes, state);
    let app = server::cors::setup(app, config.server.origins.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    server::bootstrap::app::run(app, addr, tls).await;
}
