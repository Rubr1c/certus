use std::{net::IpAddr, sync::Arc};

use axum::{
    Json,
    extract::{
        Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tokio::sync::broadcast;

use crate::{
    controller::Pagination, db::db_utils, metrics::MetricEvent,
    server::app_state::AppState,
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
        db_utils::get_request_metrics_aggregated(
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
        db_utils::get_cache_metrics_aggregated(
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
        db_utils::get_request_metrics_summary(
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

pub async fn metrics_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| {
        handle_metrics_socket(
            socket,
            state.metrics_broadcast_tx.as_ref().unwrap().clone(),
        )
    })
}

pub async fn handle_metrics_socket(
    mut socket: WebSocket,
    metrics_tx: broadcast::Sender<MetricEvent>,
) {
    let mut metrics_rx = metrics_tx.subscribe();

    loop {
        match metrics_rx.recv().await {
            Ok(metric) => match serde_json::to_string(&metric) {
                Ok(json_string) => {
                    if socket
                        .send(Message::Text(json_string.into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to serialize metric: {}", e);
                }
            },
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::error!("Warning: Client missed {} metrics", skipped);
            }
            Err(broadcast::error::RecvError::Closed) => {
                break;
            }
        }
    }
}
