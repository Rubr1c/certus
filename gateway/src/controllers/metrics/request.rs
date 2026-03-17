use std::sync::Arc;

use axum::response::IntoResponse;

use crate::{controllers, db, server::state::app_state};

use super::queries::{
    AggregateRequestQuery, INVALID_INTERVAL_MSG, RequestMetricQuery,
    parse_interval,
};

pub async fn get_request_metrics(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
    axum::extract::Query(pagination): axum::extract::Query<
        controllers::Pagination,
    >,
    axum::extract::Query(filters): axum::extract::Query<RequestMetricQuery>,
) -> impl IntoResponse {
    let ip_str = filters.ip.map(|ip| ip.to_string());

    match db::task::run(
        Arc::clone(&state.db_conn),
        move |conn| {
            db::repository::metrics::request::get(
                conn,
                filters.from.as_deref(),
                filters.to.as_deref(),
                filters.route.as_deref(),
                filters.status,
                ip_str.as_deref(),
                filters.method.as_deref(),
                pagination.page,
                pagination.per_page,
            )
        },
        "Failed to get request metrics",
        "Request metrics query task panicked",
    )
    .await
    {
        Ok(metrics) => axum::Json(metrics).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn get_request_metrics_aggregated(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
    axum::extract::Query(filters): axum::extract::Query<AggregateRequestQuery>,
) -> impl IntoResponse {
    let interval_str = filters.interval.as_deref().unwrap_or("5m");
    let interval_secs = match parse_interval(interval_str) {
        Some(seconds) => seconds,
        None => {
            return (axum::http::StatusCode::BAD_REQUEST, INVALID_INTERVAL_MSG)
                .into_response();
        }
    };

    match db::task::run(
        Arc::clone(&state.db_conn),
        move |conn| {
            db::repository::metrics::request::agg(
                conn,
                filters.from.as_deref(),
                filters.to.as_deref(),
                filters.route.as_deref(),
                filters.status,
                filters.method.as_deref(),
                interval_secs,
            )
        },
        "Failed to get aggregated request metrics",
        "Aggregated request metrics task panicked",
    )
    .await
    {
        Ok(buckets) => axum::Json(buckets).into_response(),
        Err(status) => status.into_response(),
    }
}
