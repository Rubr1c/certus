use std::{net::IpAddr, sync::Arc};

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    controller::Pagination, db::db_utils, server::app_state::AppState,
};

#[derive(Deserialize)]
pub struct BaseMetricQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub route: Option<String>,
}

#[derive(Deserialize)]
pub struct RequestMetricQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub route: Option<String>,
    pub status: Option<u16>,
    pub ip: Option<IpAddr>,
    pub method: Option<String>,
}

pub async fn get_request_metrics(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
    Query(filters): Query<RequestMetricQuery>,
) -> impl IntoResponse {
    let conn = Arc::clone(&state.db_conn);
    let ip_str = filters.ip.map(|ip| ip.to_string());

    let result = tokio::task::spawn_blocking(move || {
        let conn_guard = conn.lock();
        db_utils::get_request_metrics(
            &conn_guard,
            filters.from.as_deref(),
            filters.to.as_deref(),
            filters.route.as_deref(),
            filters.status,
            ip_str.as_deref(),
            filters.method.as_deref(),
            pagination.page,
            pagination.per_page,
        )
    })
    .await;

    match result {
        Ok(Ok(metrics)) => Json(metrics).into_response(),
        Ok(Err(e)) => {
            tracing::error!(err = ?e, "Failed to get request metrics");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Err(e) => {
            tracing::error!(err = ?e, "Request metrics query task panicked");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn get_cache_metrics(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
    Query(filters): Query<BaseMetricQuery>,
) -> impl IntoResponse {
    let conn = Arc::clone(&state.db_conn);

    let result = tokio::task::spawn_blocking(move || {
        let conn_guard = conn.lock();
        db_utils::get_cache_metrics(
            &conn_guard,
            filters.from.as_deref(),
            filters.to.as_deref(),
            filters.route.as_deref(),
            pagination.page,
            pagination.per_page,
        )
    })
    .await;

    match result {
        Ok(Ok(metrics)) => Json(metrics).into_response(),
        Ok(Err(e)) => {
            tracing::error!(err = ?e, "Failed to get cache metrics");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Err(e) => {
            tracing::error!(err = ?e, "Cache metrics query task panicked");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
