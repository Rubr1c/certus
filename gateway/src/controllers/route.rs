use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::response::IntoResponse;
use serde::Serialize;

use crate::{
    server::state::app_state,
    upstream::{health, server},
};

#[derive(Serialize)]
pub struct UpstreamInfo {
    pub address: String,
    pub healthy: bool,
    pub active_connections: usize,
    pub total_connections: usize,
    pub idle_connections: usize,
}

#[derive(Serialize)]
pub struct RouteInfo {
    pub path: String,
    pub upstreams: Vec<UpstreamInfo>,
}

#[derive(Serialize)]
pub struct UpstreamHealth {
    pub address: String,
    pub healthy: bool,
}

pub async fn get(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
) -> impl IntoResponse {
    let config = state.config.load();
    let table = state.routing_table.load();

    let routes: Vec<RouteInfo> = config
        .routes
        .iter()
        .map(|(path, route_config)| {
            let upstreams: Vec<UpstreamInfo> = route_config
                .endpoints
                .iter()
                .filter_map(|addr| {
                    table.routes.get(addr).map(|upstream| UpstreamInfo {
                        address: upstream.pool.server_addr.to_string(),
                        healthy: upstream.health_state.load(Ordering::Relaxed)
                            == server::HealthState::Alive as u8,
                        active_connections: upstream
                            .active_connctions
                            .load(Ordering::Relaxed),
                        total_connections: upstream
                            .pool
                            .total_connections
                            .load(Ordering::Relaxed),
                        idle_connections: upstream.pool.idle_connections.len(),
                    })
                })
                .collect();

            RouteInfo { path: path.clone(), upstreams }
        })
        .collect();

    axum::Json(routes)
}

pub async fn all(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
) -> impl IntoResponse {
    let table = state.routing_table.load();

    let mut results: Vec<UpstreamHealth> =
        Vec::with_capacity(table.routes.len());

    for (_, upstream) in &table.routes {
        let ok = health::health_ok(upstream).await;
        results.push(UpstreamHealth {
            address: upstream.pool.server_addr.to_string(),
            healthy: ok,
        });
    }

    axum::Json(results)
}

pub async fn one(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
    axum::extract::Path(addr): axum::extract::Path<String>,
) -> impl IntoResponse {
    let table = state.routing_table.load();

    let Some(upstream) = table.routes.get(&addr) else {
        return Err(axum::http::StatusCode::NOT_FOUND);
    };

    let ok = health::health_ok(upstream).await;

    Ok(axum::Json(UpstreamHealth { address: addr, healthy: ok }))
}
