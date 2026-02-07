use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use axum::{
    body::{Body, to_bytes},
    extract::{ConnectInfo, Request, State},
    response::IntoResponse,
};
use hyper::Method;
use matchit::Router;

use crate::{
    config::models::AuthType,
    server::{
        app_state::AppState,
        error::GatewayError,
        middleware::{
            auth,
            cache::models::{CacheKey, CachedResponse},
            handler, load_balance,
            rate_limit::TokenBucket,
        },
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
    let path = req.uri().path();
    let config = state.config.load();

    let max_tokens = config.rate_limit.max_tokens;
    let refill_rate = config.rate_limit.refill_rate;

    let mut bucket_entry = state
        .user_tokens
        .entry(addr.ip())
        .or_insert(TokenBucket::new(config.rate_limit.max_tokens));

    let now = Instant::now();
    let duration = now.duration_since(bucket_entry.last_refill).as_secs_f64();

    let tokens_to_add = duration * refill_rate;

    bucket_entry.tokens = (bucket_entry.tokens + tokens_to_add).min(max_tokens);
    bucket_entry.last_refill = now;

    tracing::info!("Checking Rate Limit");

    if bucket_entry.tokens <= 0.0 {
        tracing::info!("Checking Rate Exceeded");
        return GatewayError::RateLimited.into_response();
    }

    let ck = CacheKey {
        // store as none for now
        // needs to be extreacted from JWT if enabled else try and extract from headers
        //
        // it might be better to just set JWT here directly as a field
        user_id: None,
        user_role: None,
        path: path.to_string(),
    };
    let method = req.method().clone();

    tracing::info!("Checking cache for path: {:?}", path);
    tracing::debug!(
        "Static cache keys: {:?}",
        state.static_cache.iter().map(|e| e.key().clone()).collect::<Vec<_>>()
    );

    match state.static_cache.get(path) {
        Some(res) => {
            tracing::info!("Returning static cached response to {}", path);
            return res.clone().into_response();
        }
        None => {
            tracing::debug!("Path {:?} not found in static cache", path);
        }
    }

    match state.cache.get(&ck) {
        Some(res) => {
            if method == Method::GET {
                tracing::info!("Returning cached response to {}", path);
                return res.into_response();
            }
        }
        None => {
            tracing::info!("Response not found in cache")
        }
    }

    let router = state.router.load();
    let routes = state.routes.load();

    let matched_route_key = match router.at(&path) {
        Ok(match_result) => match_result.value,
        Err(_) => {
            return GatewayError::NotFound.into_response();
        }
    };

    let server = load_balance::p2c_pick(matched_route_key, &routes, &config);
    let upstream = routes.get(&server).expect("Upstream Should Exist").clone();
    // TODO: this should be moved up because the token weight could be higher
    //       than the actual amount and it could underflow
    //
    //       also this will not be reached on get requests because of the cache
    //       in the current impl
    bucket_entry.tokens = bucket_entry.tokens - upstream.token_weight;
    tracing::info!("Removed {} tokens from bucket", upstream.token_weight);
    tracing::info!("Remaining tokens: {:.2}", bucket_entry.tokens);

    //TODO: strip any prefix and define in config
    if let Some(ref a) = config.auth {
        if upstream.req_auth {
            let token = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "));
            match token {
                Some(t) => match &a.method {
                    AuthType::JWT { secret } => {
                        match auth::decode(t, secret) {
                            Ok(_) => {
                                //TODO: put claims in header
                            }
                            Err(e) => return e.into_response(),
                        }
                    }
                    AuthType::None => {}
                },
                None => {
                    return GatewayError::Unauthorized.into_response();
                }
            }
        }
    }

    let res = handler::handle_request(&upstream, req).await;
    match res {
        Ok(response) => {
            let (parts, body) = response.into_parts();
            let body =
                //TODO: set limit
                to_bytes(Body::new(body), usize::MAX).await.unwrap_or_default();

            let cached = CachedResponse {
                status: parts.status,
                headers: parts.headers,
                body,
            };

            let response = cached.clone().into_response();
            // only cache get requests
            if method == Method::GET {
                state.cache.insert(ck, cached);
                tracing::info!("Saved response to cache");
            }
            response
        }
        Err(e) => e.into_response(),
    }
}
