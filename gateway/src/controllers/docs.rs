use std::sync::Arc;

use crate::{ai, server::state::app_state};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

fn json_err(status: StatusCode, message: impl Into<String>) -> Response {
    (status, Json(serde_json::json!({ "error": message.into() })))
        .into_response()
}

#[inline(always)]
pub async fn generate(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
) -> Response {
    match ai::docs::generate(Arc::clone(&state.db_conn)).await {
        Ok(doc) => Json(doc).into_response(),
        Err(e) => {
            tracing::warn!(err = %e, "Documentation generation failed");
            json_err(e.status_code(), e.client_message())
        }
    }
}

#[inline(always)]
pub async fn latest(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
) -> Response {
    match ai::docs::latest(Arc::clone(&state.db_conn)).await {
        Ok(Some(doc)) => Json(doc).into_response(),
        Ok(None) => json_err(StatusCode::NOT_FOUND, "No generated docs found"),
        Err(e) => {
            tracing::warn!(err = %e, "Failed to read generated documentation");
            json_err(e.status_code(), e.client_message())
        }
    }
}
