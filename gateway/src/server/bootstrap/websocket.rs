use std::sync::Arc;

use axum::routing::any;
use tokio::sync::broadcast;

use crate::{
    cli,
    logging::LogEntryDTO,
    metrics::MetricEvent,
    server::state::app_state::AppState,
    websocket::{log_socket, metrics_socket},
};

#[inline]
pub fn run(
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
