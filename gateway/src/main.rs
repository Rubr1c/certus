use std::{net::SocketAddr, sync::Arc, time::Duration};

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

use gateway::{
    config::{
        CmdArgs,
        cfg_utils::{reload_config, watch_config},
    },
    controller::{log_controller, metrics_controller, schema_controller},
    db::{db_utils, models::ReqResSchemaDTO},
    logging::log_util::{LogChannelLayer, LogEntryDTO},
    metrics::MetricEvent,
    server::{
        app_state::{self, AppState},
        middleware::{load_balance, router},
    },
};

#[tokio::main]
async fn main() {
    let (log_tx, mut log_rx) = mpsc::channel::<LogEntryDTO>(1024);
    let (metrics_tx, mut metrics_rx) = mpsc::channel::<MetricEvent>(1024);
    let (schema_tx, mut schema_rx) = mpsc::channel::<ReqResSchemaDTO>(1024);

    let (log_broadcast_tx, _rx) = broadcast::channel::<LogEntryDTO>(1024);

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

    let args = Arc::new(CmdArgs::try_parse().unwrap());

    let config_path = args
        .config
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("certus.config.yaml");

    let log_conn = db_utils::connect_db().expect("log db");
    let metrics_conn = db_utils::connect_db().expect("metrics db");
    let schema_conn = db_utils::connect_db().expect("schema db");

    db_utils::migrate(&log_conn).expect("migrate");

    let log_conn = Arc::new(Mutex::new(log_conn));
    let metrics_conn = Arc::new(Mutex::new(metrics_conn));
    let schema_conn = Arc::new(Mutex::new(schema_conn));

    let config = reload_config(config_path).await.unwrap();

    let tls = config.tls.clone();

    let state = Arc::new(
        AppState::new(
            config,
            metrics_conn.clone(),
            metrics_tx,
            schema_tx,
            log_broadcast_tx.clone(),
        )
        .await,
    );

    let log_conn_clone = log_conn.clone();
    tokio::spawn(async move {
        let mut batch: Vec<LogEntryDTO> = Vec::with_capacity(100);

        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                Some(entry) = log_rx.recv() => {
                    batch.push(entry.clone());
                    let _ = log_broadcast_tx.send(entry);

                    if batch.len() >= 100 {
                        db_utils::flush_log_batch(&log_conn_clone, &mut batch);
                    }
                }

                _ = interval.tick() => {
                    if !batch.is_empty() {
                        db_utils::flush_log_batch(&log_conn_clone, &mut batch);
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
                   batch.push(metric);

                   if batch.len() >= 100 {
                        db_utils::flush_metric_batch(&metrics_conn_clone, &mut batch);
                   }
                },
               _ = interval.tick() => {
                   if !batch.is_empty() {
                        db_utils::flush_metric_batch(&metrics_conn_clone, &mut batch);
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
                        db_utils::flush_req_res_schema_batch(&schema_conn, &mut batch);
                    }
                },
                _ = interval.tick() => {
                    if !batch.is_empty() {
                        db_utils::flush_req_res_schema_batch(&schema_conn, &mut batch);
                    }
                }
            }
        }
    });

    let _watcher =
        match watch_config(config_path, state.clone(), args.clone()).await {
            Ok(watcher) => Some(watcher),
            Err(_) => None,
        };

    app_state::init_server_state(state.clone(), args).await;

    let config = state.config.load();
    let port = config.server.port;

    tracing::info!("Certus Gateway Running on port {}", port);
    println!("Config watcher started. Press Ctrl+C to exit.");

    let certus_routes = Router::new()
        .route("/idle", post(load_balance::set_idle))
        .route("/schemas", get(schema_controller::get_schemas))
        .route(
            "/metrics/requests",
            get(metrics_controller::get_request_metrics),
        )
        .route("/metrics/cache", get(metrics_controller::get_cache_metrics))
        .route("/ws/logs", any(log_controller::log_ws_handler));

    let mut app = Router::new()
        .nest("/_certus", certus_routes)
        .route("/{*any}", any(router::reroute))
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
