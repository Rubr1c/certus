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
        certus_routes = certus_routes.route("/ws/logs", any(log_socket::ws));
        log_broadcast_tx = Some(broadcast::channel::<LogEntryDTO>(1024).0);
        tracing::info!(path = "/ws/logs", "Enabled logs websocket");
    }

    if args.ws.contains(&cli::WebSocketType::Metrics) {
        certus_routes =
            certus_routes.route("/ws/metrics", any(metrics_socket::ws));
        metrics_broadcast_tx = Some(broadcast::channel::<MetricEvent>(1024).0);
        tracing::info!(path = "/ws/metrics", "Enabled metrics websocket");
    }

    (certus_routes, log_broadcast_tx, metrics_broadcast_tx)
}
