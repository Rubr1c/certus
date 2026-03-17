use std::sync::Arc;

use axum::routing::{get, post};

use crate::{
    controllers::{
        config_controller, log_controller, metrics, route_controller,
        schema_controller,
    },
    middleware::load_balance,
    server::state::app_state::AppState,
};

#[inline]
pub fn run() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/idle", post(load_balance::idle_queue::set_idle))
        .route("/schemas", get(schema_controller::get_schemas))
        .route("/logs", get(log_controller::get_logs))
        .route("/metrics/requests", get(metrics::get_request_metrics))
        .route("/metrics/cache", get(metrics::get_cache_metrics))
        .route(
            "/metrics/requests/aggregate",
            get(metrics::get_request_metrics_aggregated),
        )
        .route(
            "/metrics/cache/aggregate",
            get(metrics::get_cache_metrics_aggregated),
        )
        .route(
            "/metrics/requests/summary",
            get(metrics::get_request_metrics_summary),
        )
        .route("/routes", get(route_controller::get_routes))
        .route("/upstreams/health", get(route_controller::get_all_health))
        .route("/upstreams/{addr}/health", get(route_controller::get_health))
        .route(
            "/config",
            get(config_controller::get_config)
                .put(config_controller::update_config),
        )
}
