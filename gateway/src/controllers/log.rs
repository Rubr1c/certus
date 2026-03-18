use std::sync::Arc;

use axum::response::IntoResponse;
use serde::Deserialize;

use crate::{
    controllers, db, db::models::log::LogEntry, server::state::app_state,
};

#[derive(Deserialize)]
pub struct LogQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub level: Option<String>,
    pub target: Option<String>,
    pub search: Option<String>,
}

pub async fn get(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
    axum::extract::Query(pagination): axum::extract::Query<
        controllers::Pagination,
    >,
    axum::extract::Query(filters): axum::extract::Query<LogQuery>,
) -> impl IntoResponse {
    tracing::debug!(
        page = pagination.page,
        per_page = pagination.per_page,
        from = ?filters.from,
        to = ?filters.to,
        level = ?filters.level,
        target = ?filters.target,
        search = ?filters.search,
        "Querying stored logs"
    );
    match db::task::run(
        Arc::clone(&state.db_conn),
        move |conn| {
            db::repository::log::get(
                conn,
                filters.from.as_deref(),
                filters.to.as_deref(),
                filters.level.as_deref(),
                filters.target.as_deref(),
                filters.search.as_deref(),
                pagination.page,
                pagination.per_page,
            )
        },
        "Failed to get logs",
        "Log query task panicked",
    )
    .await
    {
        Ok(logs) => axum::Json::<Vec<LogEntry>>(logs).into_response(),
        Err(status) => status.into_response(),
    }
}
