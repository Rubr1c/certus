use std::sync::Arc;

use axum::response::IntoResponse;

use crate::{
    controllers, db::repository::schema_repo, server::state::app_state,
};

pub async fn get_schemas(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
    axum::extract::Query(pagination): axum::extract::Query<
        controllers::Pagination,
    >,
) -> impl IntoResponse {
    let conn = Arc::clone(&state.db_conn);

    let result = tokio::task::spawn_blocking(move || {
        let conn_guard = conn.lock();
        schema_repo::get_req_res_schemas(
            &conn_guard,
            pagination.page,
            pagination.per_page,
        )
    })
    .await;

    match result {
        Ok(Ok(schemas)) => axum::Json(schemas).into_response(),
        Ok(Err(e)) => {
            tracing::error!(err = ?e, "Failed to get schemas");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Err(e) => {
            tracing::error!(err = ?e, "Schema query task panicked");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
