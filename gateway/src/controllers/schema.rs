use std::sync::Arc;

use axum::response::IntoResponse;

use crate::{controllers, db, server::state::app_state};

pub async fn get(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
    axum::extract::Query(pagination): axum::extract::Query<
        controllers::Pagination,
    >,
) -> impl IntoResponse {
    tracing::debug!(
        page = pagination.page,
        per_page = pagination.per_page,
        "Querying stored schemas"
    );
    match db::task::run(
        Arc::clone(&state.db_conn),
        move |conn| {
            db::repository::schema::get(
                conn,
                pagination.page,
                pagination.per_page,
            )
        },
        "Failed to get schemas",
        "Schema query task panicked",
    )
    .await
    {
        Ok(schemas) => axum::Json(schemas).into_response(),
        Err(status) => status.into_response(),
    }
}
