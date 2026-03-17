use std::sync::Arc;

use axum::response::IntoResponse;

use crate::{db, server::state::app_state};

use super::queries::{INVALID_GROUP_BY_MSG, SummaryQuery, group_by};

pub async fn get(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
    axum::extract::Query(filters): axum::extract::Query<SummaryQuery>,
) -> impl IntoResponse {
    let group = filters.group_by.as_deref().unwrap_or("route");

    if !group_by(group) {
        return (axum::http::StatusCode::BAD_REQUEST, INVALID_GROUP_BY_MSG)
            .into_response();
    }

    let group_by_owned = group.to_string();

    match db::task::run(
        Arc::clone(&state.db_conn),
        move |conn| {
            db::repository::metrics::summary::get(
                conn,
                filters.from.as_deref(),
                filters.to.as_deref(),
                &group_by_owned,
            )
        },
        "Failed to get request metrics summary",
        "Request metrics summary task panicked",
    )
    .await
    {
        Ok(summary) => axum::Json(summary).into_response(),
        Err(status) => status.into_response(),
    }
}
