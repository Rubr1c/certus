use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{Router, http::HeaderValue, routing::any};
use clap::Parser;
use parking_lot::Mutex;
use tokio::{sync::mpsc, time};
use tower_http::cors::{self, CorsLayer};
use tracing_subscriber::{
    EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt,
};

use gateway::{
    config::{
        CmdArgs,
        cfg_utils::{reload_config, watch_config},
    },
    db::db_utils,
    logging::log_util::{LogChannelLayer, LogEntryDTO},
    server::{
        app_state::{self, AppState},
        middleware::router,
    },
};

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<LogEntryDTO>(1024);

    // maybe make custom writer for this to send to a channel too?
    // not sure that will make a different or not just a thought
    let console_layer = tracing_subscriber::fmt::layer().with_filter(
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info")),
    );

    let db_layer = LogChannelLayer { tx };

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

    let conn = match db_utils::connect_db() {
        Ok(c) => c,
        Err(_) => panic!("Failed to connect to db"),
    };

    match db_utils::migrate(&conn) {
        Ok(_) => (),
        Err(_) => eprintln!("Error migrating db"),
    }

    let conn = Arc::new(Mutex::new(conn));

    let state = Arc::new(AppState::new(
        reload_config(config_path).await.unwrap(),
        conn.clone(),
    ));

    let conn_clone = conn.clone();

    tokio::spawn(async move {
        let mut batch: Vec<LogEntryDTO> = Vec::with_capacity(100);

        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                Some(entry) = rx.recv() => {
                    batch.push(entry);

                    if batch.len() >= 100 {
                        db_utils::flush_batch(&conn_clone, &mut batch).await;
                    }
                }

                _ = interval.tick() => {
                    if !batch.is_empty() {
                        db_utils::flush_batch(&conn_clone, &mut batch).await;
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

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind TCP listener");

    tracing::info!("Certus Gateway Running on port {}", port);
    println!("Config watcher started. Press Ctrl+C to exit.");

    let mut app =
        Router::new().route("/{*any}", any(router::reroute)).with_state(state);

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

    let shutdown_signal = async {
        tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
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
