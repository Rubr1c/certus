use std::sync::Arc;

use axum::response::IntoResponse;

use crate::{
    config::Config,
    db,
    server::state::{self, app_state},
};

#[inline(always)]
pub async fn get(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
) -> impl IntoResponse {
    let config = state.config.load();
    tracing::trace!(
        route_count = config.routes.len(),
        "Serving config snapshot"
    );
    let value = serde_json::to_value(config.as_ref()).unwrap();
    axum::Json(value)
}

#[inline(always)]
pub async fn update(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
    axum::Json(new_config): axum::Json<Config>,
) -> impl IntoResponse {
    let new_config = Arc::new(new_config);
    tracing::info!(
        route_count = new_config.routes.len(),
        tls_enabled = new_config.tls.is_some(),
        "Updating gateway config"
    );
    let config_ref = new_config.clone();
    match db::task::run(
        Arc::clone(&state.db_conn),
        move |conn| db::repository::config::save(conn, &config_ref),
        "Failed to save config",
        "Config save task panicked",
    )
    .await
    {
        Ok(()) => {}
        Err(status) => return status.into_response(),
    }

    state.config.store(new_config);
    state::initializer::init(state.clone()).await;
    tracing::info!("Gateway config updated and state reinitialized");

    axum::http::StatusCode::OK.into_response()
}
