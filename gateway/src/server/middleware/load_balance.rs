use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, atomic::Ordering},
};

use axum::{
    body::{Body, to_bytes},
    extract::State,
    response::IntoResponse,
};
use crossbeam::queue::SegQueue;
use dashmap::DashMap;
use hyper::{Request, StatusCode};
use rand::{SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use tracing::instrument;

use crate::{
    config::RouteConfig,
    server::{app_state::AppState, upstream::UpstreamServer},
};

thread_local! {
    /// Small random number generator that has one instance per thread
    /// to avoid too many syscalls
    static THREAD_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_os_rng());
}

// only power of 2 choices for now

/// Picks randomly between 2 servers and picks the one
/// with the least load
///
/// # Arguments
///
/// * `routes` - map of all addresses to servers
/// * `target` - the config for the route targeted
/// * `config` - gateway config
#[inline]
#[instrument(name = "p2c", skip_all)]
pub fn p2c_pick<'a>(
    routes: &'a HashMap<String, Arc<UpstreamServer>>,
    target: &'a RouteConfig,
    default_server: &'a String,
) -> &'a String {
    tracing::info!("Finding endpoint");
    let endpoints = &target.endpoints;
    if endpoints.is_empty() {
        tracing::warn!("No endpoints found");
        tracing::info!("returning default server");
        return default_server;
    }

    if endpoints.len() == 1 {
        return &endpoints[0];
    }

    THREAD_RNG.with(|rng_cell| {
        let mut rng = rng_cell.borrow_mut();

        let [addr1, addr2]: [String; 2] =
            endpoints.choose_multiple_array(&mut *rng).unwrap();

        let (key1, upstream1) = routes.get_key_value(addr1.as_str()).unwrap();
        let (key2, upstream2) = routes.get_key_value(addr2.as_str()).unwrap();

        tracing::info!("Selecting server with least load");
        if upstream1.active_connctions.load(Ordering::Acquire)
            <= upstream2.active_connctions.load(Ordering::Acquire)
        {
            key1
        } else {
            key2
        }
    })
}

#[inline]
#[instrument(name = "lb", skip_all)]
pub fn run<'a>(
    routes: &'a HashMap<String, Arc<UpstreamServer>>,
    target: (&'a String, &'a RouteConfig),
    default_server: &'a String,
    idle_queue: &'a DashMap<String, SegQueue<Arc<UpstreamServer>>>,
) -> &'a String {
    if idle_queue.is_empty() {
        tracing::info!("Idle Queue is empty");
        return p2c_pick(&routes, &target.1, &default_server);
    }

    match idle_queue.get(target.0) {
        Some(q) => match q.pop() {
            Some(server) => routes
                .get_key_value(server.pool.server_addr.as_ref())
                .map(|(key, _)| key)
                .unwrap_or_else(|| {
                    tracing::info!("Idle Queue is empty");
                    p2c_pick(routes, &target.1, default_server)
                }),
            _ => {
                tracing::info!("Idle Queue is empty");
                p2c_pick(routes, &target.1, default_server)
            }
        },
        _ => p2c_pick(routes, &target.1, default_server),
    }
}

pub async fn set_idle(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let bytes = match to_bytes(req.into_body(), 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let body_string = String::from_utf8_lossy(&bytes).into_owned();

    let config = state.config.load();
    let routing_table = state.routing_table.load();
    let mut upstream: Option<Arc<UpstreamServer>> = None;

    for (addr, u_server) in &routing_table.routes {
        if *addr == body_string {
            upstream = Some(u_server.clone());
        }
    }

    for (route, route_config) in &config.routes {
        if route_config.endpoints.contains(&body_string) {
            if let Some(server) = &upstream {
                let queue = state.idle_queue.get_mut(route);
                match queue {
                    Some(q) => {
                        tracing::info!("Server pushed to idle");
                        q.push(server.clone());
                    }
                    _ => {
                        let q = SegQueue::<Arc<UpstreamServer>>::new();
                        q.push(server.clone());
                        tracing::info!("Server pushed to idle");
                        state.idle_queue.insert(route.clone(), q);
                    }
                }
            }
        }
    }

    StatusCode::OK.into_response()
}
