use std::sync::Arc;

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use tokio::sync::broadcast;

use crate::{metrics::MetricEvent, server::state::app_state};

pub async fn metrics_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<app_state::AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| {
        handle_metrics_socket(
            socket,
            // has to exist to have reached here
            state.metrics_broadcast_tx.as_ref().unwrap().clone(),
        )
    })
}

pub async fn handle_metrics_socket(
    mut socket: WebSocket,
    metrics_tx: broadcast::Sender<MetricEvent>,
) {
    let mut metrics_rx = metrics_tx.subscribe();

    loop {
        match metrics_rx.recv().await {
            Ok(metric) => match serde_json::to_string(&metric) {
                Ok(json_string) => {
                    if socket
                        .send(Message::Text(json_string.into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to serialize metric: {}", e);
                }
            },
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::error!("Warning: Client missed {} metrics", skipped);
            }
            Err(broadcast::error::RecvError::Closed) => {
                break;
            }
        }
    }
}
