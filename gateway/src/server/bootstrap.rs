use std::{net::SocketAddr, sync::Arc};

use axum::routing::{any, get, post};
use parking_lot::Mutex;
use tokio::sync::{broadcast, mpsc};
use tracing_subscriber::{
    self, EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::{
    cli,
    config::types::TLSConfig,
    controllers::{
        config_controller, log_controller, metrics_controller,
        route_controller, schema_controller,
    },
    db,
    logging::{layer::LogChannelLayer, types::LogEntryDTO},
    metrics::types::MetricEvent,
    middleware::{load_balance, pipeline},
    server::{self, shutdown, state::app_state::AppState},
    websocket::{log_socket, metrics_socket},
};

#[inline]
pub fn tracing(log_tx: mpsc::Sender<LogEntryDTO>) {
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
}

#[inline]
pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
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
        )
}

#[inline]
pub fn websocket(
    mut certus_routes: axum::Router<Arc<AppState>>,
    args: &cli::CmdArgs,
) -> (
    axum::Router<Arc<AppState>>,
    Option<broadcast::Sender<LogEntryDTO>>,
    Option<broadcast::Sender<MetricEvent>>,
) {
    let mut log_broadcast_tx = None;
    let mut metrics_broadcast_tx = None;

    if args.ws.contains(&cli::WebSocketType::Logs) {
        certus_routes =
            certus_routes.route("/ws/logs", any(log_socket::log_ws_handler));
        log_broadcast_tx = Some(broadcast::channel::<LogEntryDTO>(1024).0);
        tracing::debug!("Running log ws server");
    }

    if args.ws.contains(&cli::WebSocketType::Metrics) {
        certus_routes = certus_routes
            .route("/ws/metrics", any(metrics_socket::metrics_ws_handler));
        metrics_broadcast_tx = Some(broadcast::channel::<MetricEvent>(1024).0);
        tracing::debug!("Running metrics ws server");
    }

    (certus_routes, log_broadcast_tx, metrics_broadcast_tx)
}

#[inline]
pub fn db() -> (
    Arc<Mutex<rusqlite::Connection>>,
    Arc<Mutex<rusqlite::Connection>>,
    Arc<Mutex<rusqlite::Connection>>,
) {
    let log_conn = db::connection::connect_db().expect("log db");
    let metrics_conn = db::connection::connect_db().expect("metrics db");
    let schema_conn = db::connection::connect_db().expect("schema db");

    db::migration::migrate(&log_conn).expect("migrate");

    let log_conn = Arc::new(Mutex::new(log_conn));
    let metrics_conn = Arc::new(Mutex::new(metrics_conn));
    let schema_conn = Arc::new(Mutex::new(schema_conn));

    (log_conn, metrics_conn, schema_conn)
}

#[inline]
pub async fn app(
    app: axum::Router,
    address: SocketAddr,
    tls_config: Option<TLSConfig>,
) {
    match tls_config {
        Some(conf) => {
            server::tls::serve(app, address, server::tls::load(conf).await)
                .await;
        }
        _ => {
            let listener = tokio::net::TcpListener::bind(address)
                .await
                .expect("Failed to bind TCP listener");

            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(shutdown::create_signal())
            .await
            .unwrap();
        }
    }
}

#[inline]
pub fn router(
    interal_routes: axum::Router<Arc<AppState>>,
    app_state: Arc<AppState>,
) -> axum::Router {
    axum::Router::new()
        .nest("/_certus/api/v1", interal_routes)
        .route("/{*any}", any(pipeline::reroute))
        .with_state(app_state)
}
