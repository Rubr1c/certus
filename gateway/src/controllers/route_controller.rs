use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use std::sync::atomic::Ordering;

use crate::{
    server::state::app_state::AppState,
    upstream::{health, server::HealthState},
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

pub async fn get_routes(
    State(state): State<Arc<AppState>>,
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
                            == HealthState::Alive as u8,
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

    Json(routes)
}

pub async fn get_all_health(
    State(state): State<Arc<AppState>>,
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

    Json(results)
}

pub async fn get_health(
    State(state): State<Arc<AppState>>,
    Path(addr): Path<String>,
) -> impl IntoResponse {
    let table = state.routing_table.load();

    let Some(upstream) = table.routes.get(&addr) else {
        return Err(StatusCode::NOT_FOUND);
    };

    let ok = health::health_ok(upstream).await;

    Ok(Json(UpstreamHealth { address: addr, healthy: ok }))
}
