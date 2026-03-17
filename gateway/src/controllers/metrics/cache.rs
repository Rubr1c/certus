use std::sync::Arc;

use axum::response::IntoResponse;

use crate::{controllers, db, server::state::app_state};

use super::queries::{
    AggregateCacheQuery, BaseMetricQuery, INVALID_INTERVAL_MSG, interval,
};

pub async fn get(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
    axum::extract::Query(pagination): axum::extract::Query<
        controllers::Pagination,
    >,
    axum::extract::Query(filters): axum::extract::Query<BaseMetricQuery>,
) -> impl IntoResponse {
    match db::task::run(
        Arc::clone(&state.db_conn),
        move |conn| {
            db::repository::metrics::cache::get(
                conn,
                filters.from.as_deref(),
                filters.to.as_deref(),
                filters.route.as_deref(),
                pagination.page,
                pagination.per_page,
            )
        },
        "Failed to get cache metrics",
        "Cache metrics query task panicked",
    )
    .await
    {
        Ok(metrics) => axum::Json(metrics).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn agg(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
    axum::extract::Query(filters): axum::extract::Query<AggregateCacheQuery>,
) -> impl IntoResponse {
    let interval_str = filters.interval.as_deref().unwrap_or("5m");
    let interval_secs = match interval(interval_str) {
        Some(seconds) => seconds,
        None => {
            return (axum::http::StatusCode::BAD_REQUEST, INVALID_INTERVAL_MSG)
                .into_response();
        }
    };

    match db::task::run(
        Arc::clone(&state.db_conn),
        move |conn| {
            db::repository::metrics::cache::agg(
                conn,
                filters.from.as_deref(),
                filters.to.as_deref(),
                filters.route.as_deref(),
                interval_secs,
            )
        },
        "Failed to get aggregated cache metrics",
        "Aggregated cache metrics task panicked",
    )
    .await
    {
        Ok(buckets) => axum::Json(buckets).into_response(),
        Err(status) => status.into_response(),
    }
}
