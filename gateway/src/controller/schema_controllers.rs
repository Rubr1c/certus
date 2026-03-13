use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{db::db_utils, server::app_state::AppState};

fn default_per_page() -> u32 {
    20
}

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default)]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

pub async fn get_schemas(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> impl IntoResponse {
    let conn = Arc::clone(&state.db_conn);

    let result = tokio::task::spawn_blocking(move || {
        let conn_guard = conn.lock();
        db_utils::get_req_res_schemas(
            &conn_guard,
            pagination.page,
            pagination.per_page,
        )
    })
    .await;

    match result {
        Ok(Ok(schemas)) => Json(schemas).into_response(),
        Ok(Err(e)) => {
            tracing::error!(err = ?e, "Failed to get schemas");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Err(e) => {
            tracing::error!(err = ?e, "Schema query task panicked");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
