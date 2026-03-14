use std::sync::Arc;

use axum::{
    Json,
    extract::{
        Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tokio::sync::broadcast;

use crate::{
    controller::Pagination, db::db_utils, logging::log_util::LogEntryDTO,
    server::app_state::AppState,
};

#[derive(Deserialize)]
pub struct LogQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub level: Option<String>,
    pub target: Option<String>,
    pub search: Option<String>,
}

pub async fn get_logs(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
    Query(filters): Query<LogQuery>,
) -> impl IntoResponse {
    let conn = Arc::clone(&state.db_conn);

    let result = tokio::task::spawn_blocking(move || {
        let conn_guard = conn.lock();
        db_utils::get_log_entries(
            &conn_guard,
            filters.from.as_deref(),
            filters.to.as_deref(),
            filters.level.as_deref(),
            filters.target.as_deref(),
            filters.search.as_deref(),
            pagination.page,
            pagination.per_page,
        )
    })
    .await;

    match result {
        Ok(Ok(logs)) => Json(logs).into_response(),
        Ok(Err(e)) => {
            tracing::error!(err = ?e, "Failed to get logs");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Err(e) => {
            tracing::error!(err = ?e, "Log query task panicked");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

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
