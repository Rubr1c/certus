use std::{net::SocketAddr, sync::Arc};

use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    response::IntoResponse,
};
use hyper::{Method, header};
use matchit::Router;
use tracing::{Level, instrument};

use crate::server::{
    app_state::AppState,
    error::GatewayError,
    middleware::{
        auth,
        cache::{CacheKey, dyn_cache, static_cache},
        handler, load_balance, rate_limit,
    },
};

/// Builds a radix tree router of all the routes configured
/// and returns the router built
///
/// # Arguments
///
/// * `state` - arc of the AppState used to get the routes
pub fn build_tree(state: Arc<AppState>) -> Router<String> {
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

    router
}

/// Main axum function of the gateway that calls all the modules
/// with the request data and app state and returns a response
///
/// # Arguments
///
/// * `state` - app state injected by axum
/// * `addr` - socket address of requester
/// * `req` - request body
#[instrument(name = "router", skip_all, fields(ip = %addr.ip()))]
pub async fn reroute(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut req: Request<Body>,
) -> impl IntoResponse {
    let uri = req.uri();
    let path = uri.path();
    let config = state.config.load();
    let routing_table = state.routing_table.load();

    let matched_route_key = match routing_table.router.at(&path) {
        Ok(match_result) => match_result.value,
        Err(_) => {
            return GatewayError::NotFound.into_response();
        }
    };

    let target_route =
        config.routes.get(matched_route_key).expect("route should exist");

    let span = tracing::span!(Level::INFO, "route");
    let _span_guard = span.enter();

    match rate_limit::run(&target_route, addr.ip(), &config, &state.user_tokens)
    {
        Ok(_) => {}
        Err(err) => return err.into_response(),
    }

    let headers = req.headers();

    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            let prefix = &config.auth.prefix;
            s.strip_prefix(prefix.as_str())?.strip_prefix(' ')
        })
        .map(|s| s.to_string());

    let method = req.method().clone();

    let ck = CacheKey {
        token: token.clone(),
        path: uri
            .query()
            .map_or_else(|| path.to_string(), |q| format!("{}?{}", path, q)),
    };

    // need to put this in a fn or something and these checks are prob expensive

    //config no-cache
    let c_no_cache = target_route.no_cache;
    let cacheable_method = method == Method::GET;

    //header cache options
    let mut h_no_cache = false;
    let mut h_no_store = false;
    let mut h_private = false;
    let mut h_max_age: Option<u64> = None;

    let mut no_store = false;

    if !c_no_cache && cacheable_method {
        if let Some(cc_header) =
            headers.get(header::CACHE_CONTROL).and_then(|h| h.to_str().ok())
        {
            for part in cc_header.split(',') {
                let part = part.trim();
                match part {
                    "no-cache" => h_no_cache = true,
                    "no-store" => h_no_store = true,
                    "private" => h_private = true,
                    _ if part.starts_with("max-age=") => {
                        h_max_age = part[8..].parse::<u64>().ok();
                    }
                    _ => {}
                }
            }
        }
        no_store = c_no_cache || h_no_store || h_private || !cacheable_method;
        let no_cache = no_store || h_no_cache;

        if !no_cache {
            //can maybe combine both cache methods into one fn
            if let Some(res) =
                static_cache::try_find(&state.static_cache, path).await
            {
                return res;
            }

            if let Some(res) =
                dyn_cache::try_find(&state.cache, path, &ck).await
            {
                return res;
            }
        }
    }

    let server = load_balance::p2c_pick(
        &routing_table.routes,
        &target_route,
        &config.default_server,
    );
    let upstream = routing_table
        .routes
        .get(server)
        .expect("Upstream Should Exist")
        .clone();

    match auth::run(
        req.headers_mut(),
        token.as_deref(),
        &config,
        target_route.needs_auth,
    ) {
        Ok(_) => {}
        Err(e) => return e.into_response(),
    }

    let res = handler::handle_request(
        &upstream,
        req,
        config.connection.connect_timeout,
    )
    .await;

    match res {
        Ok(response) => {
            // dont try and save to cache only if config no cache set
            if no_store {
                return response.into_response();
            }
            return dyn_cache::try_save(
                response,
                &method,
                &state.cache,
                ck,
                h_max_age,
            )
            .await;
        }
        Err(e) => e.into_response(),
    }
}
