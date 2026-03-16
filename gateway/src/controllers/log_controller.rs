use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{db::repository::log_repo, server::state::app_state::AppState};

use super::Pagination;

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
        log_repo::get_log_entries(
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
