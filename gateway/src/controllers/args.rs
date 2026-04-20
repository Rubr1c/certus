use std::sync::Arc;

use axum::response::IntoResponse;

use crate::server::state::app_state;

#[inline(always)]
pub async fn get(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
) -> impl IntoResponse {
    axum::Json(state.args.clone())
}
