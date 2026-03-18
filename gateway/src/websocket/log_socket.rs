use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use tokio::sync::broadcast;

use crate::{logging::LogEntryDTO, server::state::app_state};

#[inline(always)]
pub async fn ws(
    ws: WebSocketUpgrade,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<SocketAddr>,
    State(state): State<Arc<app_state::AppState>>,
) -> Response {
    tracing::info!(client_ip = %addr.ip(), "Log websocket client connected");
    ws.on_upgrade(move |socket| {
        run(
            socket,
            addr,
            // has to exist to have reached here
            state.log_tx.as_ref().unwrap().clone(),
        )
    })
}

#[inline(always)]
pub async fn run(
    mut socket: WebSocket,
    addr: SocketAddr,
    log_tx: broadcast::Sender<LogEntryDTO>,
) {
    let mut log_rx = log_tx.subscribe();

    loop {
        match log_rx.recv().await {
            Ok(log_entry) => match serde_json::to_string(&log_entry) {
                Ok(json_string) => {
                    if socket
                        .send(Message::Text(json_string.into()))
                        .await
                        .is_err()
                    {
                        tracing::info!(
                            client_ip = %addr.ip(),
                            "Log websocket client disconnected"
                        );
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to serialize log entry: {}", e);
                }
            },
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::warn!(
                    client_ip = %addr.ip(),
                    skipped,
                    "Log websocket client lagged"
                );
            }
            Err(broadcast::error::RecvError::Closed) => {
                tracing::info!(
                    client_ip = %addr.ip(),
                    "Log websocket broadcast closed"
                );
                break;
            }
        }
    }
}
