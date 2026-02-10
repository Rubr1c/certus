use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    response::IntoResponse,
};
use matchit::Router;

use crate::server::{
    app_state::AppState,
    error::GatewayError,
    middleware::{
        auth,
        cache::{dyn_cache, models::CacheKey, static_cache},
        handler, load_balance, rate_limit,
    },
};

pub fn build_tree(state: Arc<AppState>) {
    let config = state.config.load();
    let route_conf = &config.routes;

    let mut router = Router::new();

    for (route, _) in route_conf {
        if let Err(e) = router.insert(route, route.clone()) {
            tracing::error!("Failed to insert route '{}': {}", route, e);
        }

        let wildcard_route = if route == "/" {
            "/{*catchall}".to_string()
        } else {
            format!("{}/{{*catchall}}", route)
        };

        if let Err(e) = router.insert(wildcard_route, route.clone()) {
            tracing::error!("Failed to insert route '{}': {}", route, e);
        }
    }

    state.router.store(Arc::new(router));
}

pub async fn reroute(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<Body>,
) -> impl IntoResponse {
    let uri = req.uri();
    let path = uri.path();
    let config = state.config.load();
    let router = state.router.load();

    let matched_route_key = match router.at(&path) {
        Ok(match_result) => match_result.value,
        Err(_) => {
            return GatewayError::NotFound.into_response();
        }
    };

    let target_route =
        config.routes.get(matched_route_key).expect("route should exist");

    match rate_limit::run(&target_route, addr.ip(), &config, &state) {
        Ok(_) => {}
        Err(err) => return err.into_response(),
    }

    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    let ck = CacheKey {
        token: token.map(|s| s.to_string()),
        path: uri.query().map_or_else(
            || path.to_string(),
            |q| format!("{}?{}", path, q)
        ),
    };

    let method = req.method().clone();

    match static_cache::try_find(&state, path) {
        Some(res) => return res,
        _ => {}
    }

    match dyn_cache::try_find(&state, path, &ck, &method) {
        Some(res) => return res,
        _ => {}
    }

    let routes = state.routes.load();

    let server = load_balance::p2c_pick(&routes, &target_route, &config);
    let upstream = routes.get(&server).expect("Upstream Should Exist").clone();

    match auth::run(&upstream, &config, token) {
        Ok(_) => {}
        Err(e) => return e.into_response(),
    }

    let res = handler::handle_request(&upstream, req).await;
    match res {
        Ok(response) => {
            return dyn_cache::try_save(response, &method, &state, ck).await;
        }
        Err(e) => e.into_response(),
    }
}
