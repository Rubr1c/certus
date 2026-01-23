use std::sync::Arc;

use axum::{
    body::{Body, to_bytes},
    extract::{Request, State},
    response::IntoResponse,
};
use matchit::Router;

use crate::server::{
    app_state::AppState,
    error::GatewayError,
    load_balancing::balancing,
    models::{CacheKey, CachedResponse},
    request::requests,
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
    req: Request<Body>,
) -> impl IntoResponse {
    let path = req.uri().path();
    let ck = CacheKey {
        method: req.method().clone(),
        //store as none for now
        user_id: None,
        user_role: None,
        path: path.to_string(),
    };

    match state.cache.get(&ck) {
        Some(res) => {
            tracing::info!("returning cached response");
            return res.into_response();
        }
        None => {}
    }

    let router = state.router.load();
    let routes = state.routes.load();
    let config = state.config.load();

    let matched_route_key = match router.at(&path) {
        Ok(match_result) => match_result.value,
        Err(_) => {
            return GatewayError::NotFound.into_response();
        }
    };

    let server = balancing::p2c_pick(matched_route_key, &routes, &config);
    let upstream = routes.get(&server).expect("Upstream Should Exist").clone();

    let res = requests::handle_request(&upstream, req).await;
    match res {
        Ok(response) => {
            let (parts, body) = response.into_parts();
            let body =
                to_bytes(Body::new(body), usize::MAX).await.unwrap_or_default();

            let cached = CachedResponse {
                status: parts.status,
                headers: parts.headers,
                body,
            };

            let response = cached.clone().into_response();
            state.cache.insert(ck, cached);

            response
        }
        Err(e) => e.into_response(),
    }
}
