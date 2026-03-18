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

#[inline(always)]
pub async fn run() {
    let (log_tx, log_rx) = mpsc::channel::<LogEntryDTO>(1024);
    let (metrics_tx, metrics_rx) = mpsc::channel::<MetricEvent>(1024);
    let (schema_tx, schema_rx) = mpsc::channel::<ReqResSchemaDTO>(1024);

    server::bootstrap::tracing::run(log_tx);

    let args = Arc::new(cli::CmdArgs::parse());

    let config_path = args.config.as_str();

    tracing::info!(
        config_path,
        save_config = args.save,
        websocket_servers = ?args.ws,
        "Starting Certus gateway"
    );

    let certus_routes = server::bootstrap::routes::run();
    let (certus_routes, log_broadcast_tx, metrics_broadcast_tx) =
        server::bootstrap::websocket::run(certus_routes, args.as_ref());

    let (log_conn, metrics_conn, schema_conn) = server::bootstrap::db::run();

    let config = match config::parser::reload(config_path).await {
        Ok(config) => config,
        Err(err) => {
            tracing::error!(
                config_path,
                err = ?err,
                "Failed to load gateway config"
            );
            panic!("failed to load gateway config");
        }
    };
    tracing::info!(
        config_path,
        route_count = config.routes.len(),
        tls_enabled = config.tls.is_some(),
        "Loaded gateway config"
    );

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

    tracing::info!("Application state initialized");

    tasks::log_flusher::run(log_conn.clone(), log_rx, log_broadcast_tx).await;
    tracing::info!("Log flusher task started");

    tasks::metric_flusher::run(
        metrics_conn.clone(),
        metrics_rx,
        metrics_broadcast_tx,
    )
    .await;
    tracing::info!("Metric flusher task started");

    tasks::schema_flusher::run(schema_conn, schema_rx).await;
    tracing::info!("Schema flusher task started");

    tasks::health_reaper::run(state.clone()).await;
    tracing::info!("Health reaper tasks started");

    let _watcher =
        match config::watcher::watch(config_path, state.clone()).await {
            Ok(watcher) => {
                tracing::info!(config_path, "Config watcher started");
                Some(watcher)
            }
            Err(err) => {
                tracing::warn!(
                    config_path,
                    err = ?err,
                    "Failed to start config watcher"
                );
                None
            }
        };

    initializer::init(state.clone()).await;
    tracing::info!("Gateway state initialization complete");

    let config = state.config.load();
    let port = config.server.port;

    tracing::info!(port, "Certus gateway ready");

    let app = server::bootstrap::router::run(certus_routes, state);
    let app = server::cors::setup(app, config.server.origins.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    server::bootstrap::app::run(app, addr, tls).await;
}
