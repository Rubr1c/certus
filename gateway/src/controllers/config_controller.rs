use std::sync::Arc;

use axum::response::IntoResponse;

use crate::{
    config::types,
    db::repository::config_repo,
    server::state::{self, app_state},
};

pub async fn get_config(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
) -> impl IntoResponse {
    let config = state.config.load();
    let value = serde_json::to_value(config.as_ref()).unwrap();
    axum::Json(value)
}

pub async fn update_config(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
    axum::Json(new_config): axum::Json<types::Config>,
) -> impl IntoResponse {
    let new_config = Arc::new(new_config);
    let config_ref = new_config.clone();
    let conn = Arc::clone(&state.db_conn);

    let result = tokio::task::spawn_blocking(move || {
        let conn_guard = conn.lock();
        config_repo::save_config(&conn_guard, &config_ref)
    })
    .await;

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            tracing::error!(err = ?e, "Failed to save config");
            return axum::http::StatusCode::INTERNAL_SERVER_ERROR
                .into_response();
        }
        Err(e) => {
            tracing::error!(err = ?e, "Config save task panicked");
            return axum::http::StatusCode::INTERNAL_SERVER_ERROR
                .into_response();
        }
    }

    state.config.store(new_config);
    state::initializer::init_server_state(state.clone()).await;

    axum::http::StatusCode::OK.into_response()
}
