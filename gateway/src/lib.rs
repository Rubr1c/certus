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

use std::{
    net::SocketAddr,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use axum::{
    Router,
    http::HeaderValue,
    routing::{any, get, post},
};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use parking_lot::Mutex;
use tokio::{
    sync::{broadcast, mpsc},
    time,
};
use tower_http::cors::{self, CorsLayer};
use tracing_subscriber::{
    EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::{
    cli::{CmdArgs, WebSocketType},
    controllers::{
        config_controller, log_controller, metrics_controller,
        route_controller, schema_controller,
    },
    logging::{layer::LogChannelLayer, types::LogEntryDTO},
    metrics::types::MetricEvent,
    middleware::{load_balance, pipeline, router},
    schema::types::ReqResSchemaDTO,
    server::state::{self, app_state::AppState, routing_table::RoutingTable},
    upstream::{health, server::HealthState},
    websocket::{log_socket, metrics_socket},
};

pub async fn run() {
    let (log_tx, mut log_rx) = mpsc::channel::<LogEntryDTO>(1024);
    let (metrics_tx, mut metrics_rx) = mpsc::channel::<MetricEvent>(1024);
    let (schema_tx, mut schema_rx) = mpsc::channel::<ReqResSchemaDTO>(1024);

    let mut log_broadcast_tx: Option<broadcast::Sender<LogEntryDTO>> = None;
    let mut metrics_broadcast_tx: Option<broadcast::Sender<MetricEvent>> = None;

    // maybe make custom writer for this to send to a channel too?
    // not sure that will make a different or not just a thought
    let console_layer = tracing_subscriber::fmt::layer().with_filter(
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info")),
    );

    let db_layer =
        LogChannelLayer { tx: log_tx }.with_filter(EnvFilter::new("debug"));

    let _ = tracing_subscriber::registry()
        .with(console_layer)
        .with(db_layer)
        .try_init();

    let args = Arc::new(CmdArgs::parse());

    let config_path = args.config.as_str();

    let mut certus_routes = Router::new()
        .route("/idle", post(load_balance::idle_queue::set_idle))
        .route("/schemas", get(schema_controller::get_schemas))
        .route("/logs", get(log_controller::get_logs))
        .route(
            "/metrics/requests",
            get(metrics_controller::get_request_metrics),
        )
        .route("/metrics/cache", get(metrics_controller::get_cache_metrics))
        .route(
            "/metrics/requests/aggregate",
            get(metrics_controller::get_request_metrics_aggregated),
        )
        .route(
            "/metrics/cache/aggregate",
            get(metrics_controller::get_cache_metrics_aggregated),
        )
        .route(
            "/metrics/requests/summary",
            get(metrics_controller::get_request_metrics_summary),
        )
        .route("/routes", get(route_controller::get_routes))
        .route("/upstreams/health", get(route_controller::get_all_health))
        .route("/upstreams/{addr}/health", get(route_controller::get_health))
        .route(
            "/config",
            get(config_controller::get_config)
                .put(config_controller::update_config),
        );

    if args.ws.contains(&WebSocketType::Logs) {
        certus_routes =
            certus_routes.route("/ws/logs", any(log_socket::log_ws_handler));
        log_broadcast_tx = Some(broadcast::channel::<LogEntryDTO>(1024).0);
        tracing::debug!("Running log ws server");
    }

    if args.ws.contains(&WebSocketType::Metrics) {
        certus_routes = certus_routes
            .route("/ws/metrics", any(metrics_socket::metrics_ws_handler));
        metrics_broadcast_tx = Some(broadcast::channel::<MetricEvent>(1024).0);
        tracing::debug!("Running metrics ws server");
    }

    let log_conn = db::connection::connect_db().expect("log db");
    let metrics_conn = db::connection::connect_db().expect("metrics db");
    let schema_conn = db::connection::connect_db().expect("schema db");

    db::migration::migrate(&log_conn).expect("migrate");

    let log_conn = Arc::new(Mutex::new(log_conn));
    let metrics_conn = Arc::new(Mutex::new(metrics_conn));
    let schema_conn = Arc::new(Mutex::new(schema_conn));

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

    // move all these tasks somewhere else

    let log_conn_clone = log_conn.clone();
    tokio::spawn(async move {
        let mut batch: Vec<LogEntryDTO> = Vec::with_capacity(100);

        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                Some(entry) = log_rx.recv() => {
                    batch.push(entry.clone());
                    if let Some(tx) = &log_broadcast_tx {
                        let _ = tx.send(entry);
                    }

                    if batch.len() >= 100 {
                        db::flush::flush_log_batch(&log_conn_clone, &mut batch);
                    }
                }

                _ = interval.tick() => {
                    if !batch.is_empty() {
                        db::flush::flush_log_batch(&log_conn_clone, &mut batch);
                    }
                }
            }
        }
    });

    let metrics_conn_clone = metrics_conn.clone();
    tokio::spawn(async move {
        let mut batch: Vec<MetricEvent> = Vec::with_capacity(100);

        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                Some(metric) = metrics_rx.recv() => {
                   if let Some(tx) = &metrics_broadcast_tx {
                       let _ = tx.send(metric.clone());
                   }
                   batch.push(metric);

                   if batch.len() >= 100 {
                        db::flush::flush_metric_batch(&metrics_conn_clone, &mut batch);
                   }
                },
               _ = interval.tick() => {
                   if !batch.is_empty() {
                        db::flush::flush_metric_batch(&metrics_conn_clone, &mut batch);
                   }
                }
            }
        }
    });

    tokio::spawn(async move {
        let mut batch: Vec<ReqResSchemaDTO> = Vec::with_capacity(100);

        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                Some(schema) = schema_rx.recv() => {
                    batch.push(schema);

                    if batch.len() >= 100 {
                        db::flush::flush_req_res_schema_batch(&schema_conn, &mut batch);
                    }
                },
                _ = interval.tick() => {
                    if !batch.is_empty() {
                        db::flush::flush_req_res_schema_batch(&schema_conn, &mut batch);
                    }
                }
            }
        }
    });

    let state_clone = state.clone();
    tokio::spawn(async move {
        loop {
            //TODO: make configable
            tokio::time::sleep(Duration::from_secs(5)).await;

            let table = state_clone.routing_table.load();
            let dead_addrs: Vec<String> = table
                .routes
                .iter()
                .filter(|(_, upstream)| {
                    upstream.health_state.load(Ordering::Acquire)
                        == HealthState::Dead as u8
                })
                .map(|(addr, _)| addr.clone())
                .collect();

            if dead_addrs.is_empty() {
                continue;
            }

            for addr in &dead_addrs {
                tracing::warn!(server = %addr, "Removing dead upstream");
            }

            let mut new_routes = table.routes.clone();
            for addr in &dead_addrs {
                new_routes.remove(addr);
            }

            let new_router = router::build_tree(state_clone.clone());
            let new_table =
                RoutingTable { router: new_router, routes: new_routes };
            state_clone.routing_table.store(Arc::new(new_table));

            for entry in state_clone.idle_queue.iter_mut() {
                let queue = entry.value();
                let len = queue.len();
                for _ in 0..len {
                    if let Some(upstream) = queue.pop() {
                        if upstream.health_state.load(Ordering::Acquire)
                            != HealthState::Dead as u8
                        {
                            queue.push(upstream);
                        }
                    }
                }
            }
        }
    });

    let state_clone = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;

            let table = state_clone.routing_table.load();
            for (addr, upstream) in &table.routes {
                if !health::health_ok(upstream).await {
                    tracing::warn!(server = %addr, "Health check failed");
                }
            }
        }
    });

    let _watcher =
        match config::watcher::watch_config(config_path, state.clone()).await {
            Ok(watcher) => Some(watcher),
            Err(_) => None,
        };

    state::initializer::init_server_state(state.clone()).await;

    let config = state.config.load();
    let port = config.server.port;

    tracing::info!("Certus Gateway Running on port {}", port);
    println!("Config watcher started. Press Ctrl+C to exit.");

    let mut app = Router::new()
        .nest("/_certus/api/v1", certus_routes)
        .route("/{*any}", any(pipeline::reroute))
        .with_state(state);

    let origins = config.server.origins.clone();

    //TODO: make emit logs
    app = if origins.is_empty() {
        tracing::warn!("No cors set allowing from all origins");
        app.layer(CorsLayer::new().allow_origin(cors::Any))
    } else {
        let parsed_origins: Vec<HeaderValue> = origins
            .iter()
            .map(|ip| ip.parse().expect("Invalid Origin IP"))
            .collect();

        app.layer(CorsLayer::new().allow_origin(parsed_origins))
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    match tls {
        Some(conf) => {
            let tls_conf =
                RustlsConfig::from_pem_file(&conf.cert_path, &conf.key_path)
                    .await
                    .expect("Invalid TLS");

            let handle = axum_server::Handle::new();
            let shutdown_handle = handle.clone();

            tokio::spawn(async move {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to listen for Ctrl+C");
                tracing::info!("\nShutting down...");
                shutdown_handle.graceful_shutdown(None);
            });

            axum_server::bind_rustls(addr, tls_conf)
                .handle(handle)
                .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                .await
                .unwrap();
        }
        _ => {
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .expect("Failed to bind TCP listener");

            let shutdown_signal = async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to listen for Ctrl+C");
                tracing::info!("\nShutting down...");
            };

            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(shutdown_signal)
            .await
            .unwrap();
        }
    }
}
