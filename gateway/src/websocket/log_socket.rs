use std::sync::Arc;

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use tokio::sync::broadcast;

use crate::{logging::types::LogEntryDTO, server::state::app_state::AppState};

pub async fn log_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| {
        handle_log_socket(
            socket,
            // has to exist to have reached here
            state.log_tx.as_ref().unwrap().clone(),
        )
    })
}

pub async fn handle_log_socket(
    mut socket: WebSocket,
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
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to serialize log entry: {}", e);
                }
            },
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::error!("Warning: Client missed {} logs", skipped);
            }
            Err(broadcast::error::RecvError::Closed) => {
                break;
            }
        }
    }
}
