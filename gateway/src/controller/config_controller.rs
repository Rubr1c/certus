use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    config::Config,
    db::db_utils,
    server::app_state::{self, AppState},
};

pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let config = state.config.load();
    let value = serde_json::to_value(config.as_ref()).unwrap();
    Json(value)
}

pub async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(new_config): Json<Config>,
) -> impl IntoResponse {
    let new_config = Arc::new(new_config);
    let config_ref = new_config.clone();
    let conn = Arc::clone(&state.db_conn);

    let result = tokio::task::spawn_blocking(move || {
        let conn_guard = conn.lock();
        db_utils::save_config(&conn_guard, &config_ref)
    })
    .await;

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            tracing::error!(err = ?e, "Failed to save config");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        Err(e) => {
            tracing::error!(err = ?e, "Config save task panicked");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    state.config.store(new_config);
    app_state::init_server_state(state.clone()).await;

    StatusCode::OK.into_response()
}
