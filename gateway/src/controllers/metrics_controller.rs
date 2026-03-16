use std::{net::IpAddr, sync::Arc};

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{db::repository::metrics_repo, server::state::app_state::AppState};

use super::Pagination;

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

#[derive(Deserialize)]
pub struct AggregateRequestQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub route: Option<String>,
    pub status: Option<u16>,
    pub method: Option<String>,
    pub interval: Option<String>,
}

#[derive(Deserialize)]
pub struct AggregateCacheQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub route: Option<String>,
    pub interval: Option<String>,
}

#[derive(Deserialize)]
pub struct SummaryQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub group_by: Option<String>,
}

fn parse_interval(s: &str) -> Option<i64> {
    match s {
        "1m" => Some(60),
        "5m" => Some(300),
        "15m" => Some(900),
        "30m" => Some(1800),
        "1h" => Some(3600),
        "6h" => Some(21600),
        "1d" => Some(86400),
        _ => None,
    }
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
        metrics_repo::get_request_metrics(
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
        metrics_repo::get_cache_metrics(
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

pub async fn get_request_metrics_aggregated(
    State(state): State<Arc<AppState>>,
    Query(filters): Query<AggregateRequestQuery>,
) -> impl IntoResponse {
    let interval_str = filters.interval.as_deref().unwrap_or("5m");
    let interval_secs = match parse_interval(interval_str) {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "Invalid interval. Use: 1m, 5m, 15m, 30m, 1h, 6h, 1d",
            )
                .into_response();
        }
    };

    let conn = Arc::clone(&state.db_conn);

    let result = tokio::task::spawn_blocking(move || {
        let conn_guard = conn.lock();
        metrics_repo::get_request_metrics_aggregated(
            &conn_guard,
            filters.from.as_deref(),
            filters.to.as_deref(),
            filters.route.as_deref(),
            filters.status,
            filters.method.as_deref(),
            interval_secs,
        )
    })
    .await;

    match result {
        Ok(Ok(buckets)) => Json(buckets).into_response(),
        Ok(Err(e)) => {
            tracing::error!(err = ?e, "Failed to get aggregated request metrics");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Err(e) => {
            tracing::error!(err = ?e, "Aggregated request metrics task panicked");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn get_cache_metrics_aggregated(
    State(state): State<Arc<AppState>>,
    Query(filters): Query<AggregateCacheQuery>,
) -> impl IntoResponse {
    let interval_str = filters.interval.as_deref().unwrap_or("5m");
    let interval_secs = match parse_interval(interval_str) {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "Invalid interval. Use: 1m, 5m, 15m, 30m, 1h, 6h, 1d",
            )
                .into_response();
        }
    };

    let conn = Arc::clone(&state.db_conn);

    let result = tokio::task::spawn_blocking(move || {
        let conn_guard = conn.lock();
        metrics_repo::get_cache_metrics_aggregated(
            &conn_guard,
            filters.from.as_deref(),
            filters.to.as_deref(),
            filters.route.as_deref(),
            interval_secs,
        )
    })
    .await;

    match result {
        Ok(Ok(buckets)) => Json(buckets).into_response(),
        Ok(Err(e)) => {
            tracing::error!(err = ?e, "Failed to get aggregated cache metrics");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Err(e) => {
            tracing::error!(err = ?e, "Aggregated cache metrics task panicked");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn get_request_metrics_summary(
    State(state): State<Arc<AppState>>,
    Query(filters): Query<SummaryQuery>,
) -> impl IntoResponse {
    let group_by = filters.group_by.as_deref().unwrap_or("route");

    match group_by {
        "route" | "method" | "status" | "upstream" => {}
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                "Invalid group_by. Use: route, method, status, upstream",
            )
                .into_response();
        }
    }

    let conn = Arc::clone(&state.db_conn);
    let group_by_owned = group_by.to_string();

    let result = tokio::task::spawn_blocking(move || {
        let conn_guard = conn.lock();
        metrics_repo::get_request_metrics_summary(
            &conn_guard,
            filters.from.as_deref(),
            filters.to.as_deref(),
            &group_by_owned,
        )
    })
    .await;

    match result {
        Ok(Ok(summary)) => Json(summary).into_response(),
        Ok(Err(e)) => {
            tracing::error!(err = ?e, "Failed to get request metrics summary");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Err(e) => {
            tracing::error!(err = ?e, "Request metrics summary task panicked");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
