use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use tokio::sync::broadcast;

use crate::{metrics::MetricEvent, server::state::app_state};

pub async fn ws(
    ws: WebSocketUpgrade,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<SocketAddr>,
    State(state): State<Arc<app_state::AppState>>,
) -> Response {
    tracing::info!(client_ip = %addr.ip(), "Metrics websocket client connected");
    ws.on_upgrade(move |socket| {
        run(
            socket,
            addr,
            // has to exist to have reached here
            state.metrics_broadcast_tx.as_ref().unwrap().clone(),
        )
    })
}

pub async fn run(
    mut socket: WebSocket,
    addr: SocketAddr,
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
                        tracing::info!(
                            client_ip = %addr.ip(),
                            "Metrics websocket client disconnected"
                        );
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to serialize metric: {}", e);
                }
            },
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::warn!(
                    client_ip = %addr.ip(),
                    skipped,
                    "Metrics websocket client lagged"
                );
            }
            Err(broadcast::error::RecvError::Closed) => {
                tracing::info!(
                    client_ip = %addr.ip(),
                    "Metrics websocket broadcast closed"
                );
                break;
            }
        }
    }
}
