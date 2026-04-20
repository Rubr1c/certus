use std::sync::Arc;

use axum::routing::{get, post};

use crate::{
    controllers::{args, config, docs, log, metrics, route, schema},
    middleware::load_balance,
    server::state::app_state::AppState,
};

#[inline(always)]
pub fn run() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/idle", post(load_balance::idle_queue::set_idle))
        .route("/schemas", get(schema::get))
        .route("/docs", get(docs::latest))
        .route("/docs/generate", post(docs::generate))
        .route("/logs", get(log::get))
        .route("/metrics/requests", get(metrics::request::get))
        .route("/metrics/cache", get(metrics::cache::get))
        .route("/metrics/requests/aggregate", get(metrics::request::agg))
        .route("/metrics/cache/aggregate", get(metrics::cache::agg))
        .route("/metrics/requests/summary", get(metrics::summary::get))
        .route("/routes", get(route::get))
        .route("/upstreams/health", get(route::all))
        .route("/upstreams/{addr}/health", get(route::one))
        .route("/config", get(config::get).put(config::update))
        .route("/args", get(args::get))
}
