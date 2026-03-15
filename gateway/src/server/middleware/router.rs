use std::{
    borrow::Cow,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    response::IntoResponse,
};
use chrono::Utc;
use hyper::{
    Method,
    header::{self, CONTENT_LENGTH, FORWARDED, HOST},
};
use matchit::Router;
use tokio::time::Instant;
use tracing::{Level, instrument};

use crate::{
    config::RateLimitKey,
    db::models::ReqResSchemaDTO,
    metrics::{
        CacheMetric, CacheResult, EarlyExit, MetricEvent, RequestMetric,
    },
    server::{
        app_state::AppState,
        error::GatewayError,
        middleware::{
            auth,
            cache::{CacheKey, dyn_cache, static_cache},
            handler, load_balance,
            rate_limit::{self, TokenBucketKey},
        },
        upstream::HttpVersion,
    },
};

/// Builds a radix tree router of all the routes configured
/// and returns the router built
///
/// # Arguments
///
/// * `state` - arc of the AppState used to get the routes
pub fn build_tree(state: Arc<AppState>) -> Router<Arc<str>> {
    let config = state.config.load();
    let route_conf = &config.routes;

    let mut router = Router::new();

    for (route, _) in route_conf {
        if let Err(e) = router.insert(route, Arc::<str>::from(route.as_str())) {
            tracing::error!("Failed to insert route '{}': {}", route, e);
        }

        let wildcard_route = if route == "/" {
            "/{*catchall}".to_string()
        } else {
            format!("{}/{{*catchall}}", route)
        };

        if let Err(e) =
            router.insert(wildcard_route, Arc::<str>::from(route.as_str()))
        {
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
    let start = Instant::now();

    let path: Arc<str> = req.uri().path().into();
    let query: Option<Arc<str>> = req.uri().query().map(Arc::from);
    let config = state.config.load();
    let routing_table = state.routing_table.load();

    let matched_route_key = match routing_table.router.at(&path) {
        Ok(match_result) => match_result.value,
        Err(_) => {
            return GatewayError::NotFound.into_response();
        }
    };

    let target_route = config
        .routes
        .get_key_value(matched_route_key.as_ref())
        .expect("route should exist");

    let span = tracing::span!(Level::INFO, "route");
    let _span_guard = span.enter();

    let headers = req.headers();
    let ip = headers
        .get(FORWARDED)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.parse::<IpAddr>().unwrap_or(addr.ip()))
        .unwrap_or(addr.ip());

    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            let prefix = &config.auth.prefix;
            s.strip_prefix(prefix.as_str())?.strip_prefix(' ')
        })
        .map(str::to_owned);

    let bytes_in = headers
        .get(CONTENT_LENGTH)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let token_bucket_key = match &config.rate_limit.key {
        RateLimitKey::Ip => TokenBucketKey::Ip(ip),
        RateLimitKey::Token => match &token {
            Some(token) => TokenBucketKey::Token(Cow::Borrowed(token.as_str())),
            _ => TokenBucketKey::Ip(ip),
        },
        RateLimitKey::Header(header) => match headers.get(header.as_str()) {
            Some(val) => TokenBucketKey::Header(
                Cow::Borrowed(header.as_str()),
                val.clone(),
            ),
            _ => TokenBucketKey::Ip(ip),
        },
    };

    match rate_limit::run(
        &target_route.1,
        token_bucket_key,
        &config,
        &state.user_tokens,
    )
    .await
    {
        Ok(_) => {}
        Err(err) => {
            let duration = start.elapsed().as_millis() as u64;
            let _ = state.metrics_tx.try_send(MetricEvent::Request(
                RequestMetric {
                    route: Arc::clone(matched_route_key),
                    status_code: 429,
                    timestamp: Utc::now(),
                    duration_total_ms: duration,
                    duration_upstream_ms: 0,
                    bytes_in,
                    bytes_out: 0,
                    client_ip: ip,
                    method: req.method().clone(),
                    upstream_addr: None,
                    early_exit: Some(EarlyExit::RateLimited),
                },
            ));
            return err.into_response();
        }
    }

    let method = req.method().clone();
    let ck = CacheKey {
        token: token.as_deref().map(Cow::Borrowed),
        path: query.as_ref().map_or_else(
            || Cow::Borrowed(&*path),
            |q| Cow::Owned(format!("{}?{}", path, q)),
        ),
    };
    // need to put this in a fn or something and these checks are prob expensive

    //config no-cache
    let c_no_cache = target_route.1.no_cache;
    let cacheable_method = method == Method::GET;

    //TODO: stale-while-revalidate, stale-if-error
    //header cache options
    let mut h_no_cache = false;
    let mut h_no_store = false;
    let mut h_private = false;
    let mut h_public = false;
    let mut h_max_age: Option<u64> = None;
    let mut h_s_max_age: Option<u64> = None;

    let mut no_store = c_no_cache || !cacheable_method;

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
                    "public" => h_public = true,
                    _ if part.starts_with("max-age=") => {
                        h_max_age = part[8..].parse::<u64>().ok();
                    }
                    _ if part.starts_with("s-maxage") => {
                        h_s_max_age = part[9..].parse::<u64>().ok();
                    }
                    _ => {}
                }
            }

            if h_s_max_age.is_some() {
                h_max_age = h_s_max_age;
            }
        }

        no_store = c_no_cache
            || h_no_store
            || h_private
            || !cacheable_method
            || (token.is_some() && !h_public);
        let no_cache = no_store || h_no_cache;

        if !no_cache {
            //can maybe combine both cache methods into one fn
            if let Some(res) =
                static_cache::try_find(&state.static_cache, &path).await
            {
                let metric = CacheMetric {
                    route: Arc::clone(matched_route_key),
                    timestamp: Utc::now(),
                    result: CacheResult::Hit,
                };
                let _ = state.metrics_tx.try_send(MetricEvent::Cache(metric));

                return res;
            }

            match dyn_cache::try_find(&state.cache, &path, &ck).await {
                Some(res) => {
                    let metric = CacheMetric {
                        route: Arc::clone(matched_route_key),
                        timestamp: Utc::now(),
                        result: CacheResult::Hit,
                    };
                    let _ =
                        state.metrics_tx.try_send(MetricEvent::Cache(metric));

                    return res;
                }
                _ => {
                    let metric = CacheMetric {
                        route: Arc::clone(matched_route_key),
                        timestamp: Utc::now(),
                        result: CacheResult::Miss,
                    };
                    let _ =
                        state.metrics_tx.try_send(MetricEvent::Cache(metric));
                }
            }
        }
    } else {
        let metric = CacheMetric {
            route: Arc::clone(matched_route_key),
            timestamp: Utc::now(),
            result: CacheResult::Bypass,
        };
        let _ = state.metrics_tx.try_send(MetricEvent::Cache(metric));
    }

    let server = load_balance::run(
        &routing_table.routes,
        (&target_route.0, &target_route.1),
        &config.default_server,
        &state.idle_queue,
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
        target_route.1.needs_auth,
    ) {
        Ok(_) => {}
        Err(e) => {
            let duration = start.elapsed().as_millis() as u64;
            let _ = state.metrics_tx.try_send(MetricEvent::Request(
                RequestMetric {
                    route: Arc::clone(matched_route_key),
                    status_code: 401,
                    timestamp: Utc::now(),
                    duration_total_ms: duration,
                    duration_upstream_ms: 0,
                    bytes_in,
                    bytes_out: 0,
                    client_ip: ip,
                    method: method.clone(),
                    upstream_addr: None,
                    early_exit: Some(EarlyExit::Unauthorized),
                },
            ));
            return e.into_response();
        }
    }

    if matches!(target_route.1.http_version, HttpVersion::HTTP1) {
        let pq =
            req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
        *req.uri_mut() = pq.parse().expect("valid path_and_query");

        if !req.headers().contains_key(HOST) {
            req.headers_mut().insert(
                HOST,
                upstream
                    .pool
                    .hostname
                    .parse()
                    .expect("upstream hostname is valid header value"),
            );
        }
    }

    let req_headers = req.headers().clone();

    let upstream_start = std::time::Instant::now();

    let res = handler::handle_request(
        &upstream,
        req,
        config.connection.connect_timeout,
    )
    .await;

    let duration_upstream_ms = upstream_start.elapsed().as_millis() as u64;

    match res {
        Ok(response) => {
            let res_headers = response.headers().clone();
            let status_code = response.status();

            let bytes_out = response
                .headers()
                .get(CONTENT_LENGTH)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            if no_store {
                let _ = state.schema_tx.try_send(ReqResSchemaDTO {
                    full_path: Arc::clone(&path),
                    method: method.clone(),
                    query_params: query.as_ref().map(Arc::clone),
                    status_code,
                    req_headers,
                    res_headers,
                    body_schema: None,
                });

                let duration = start.elapsed().as_millis() as u64;
                let event = MetricEvent::Request(RequestMetric {
                    route: Arc::clone(matched_route_key),
                    status_code: status_code.as_u16(),
                    timestamp: Utc::now(),
                    duration_total_ms: duration,
                    duration_upstream_ms,
                    bytes_in,
                    bytes_out,
                    client_ip: ip,
                    method: method.clone(),
                    upstream_addr: Some(Arc::clone(&upstream.pool.server_addr)),
                    early_exit: None,
                });

                let _ = state.metrics_tx.try_send(event);

                return response.into_response();
            }
            let (res, body_schema) = dyn_cache::try_save(
                response,
                &method,
                &state.cache,
                ck,
                h_max_age,
                config.cache.max_size,
            )
            .await;

            let _ = state.schema_tx.try_send(ReqResSchemaDTO {
                full_path: Arc::clone(&path),
                method: method.clone(),
                query_params: query.as_ref().map(Arc::clone),
                status_code,
                req_headers,
                res_headers,
                body_schema,
            });

            let duration = start.elapsed().as_millis() as u64;
            let event = MetricEvent::Request(RequestMetric {
                route: Arc::clone(matched_route_key),
                status_code: res.status().as_u16(),
                timestamp: Utc::now(),
                duration_total_ms: duration,
                duration_upstream_ms,
                bytes_in,
                bytes_out,
                client_ip: ip,
                method: method.clone(),
                upstream_addr: Some(Arc::clone(&upstream.pool.server_addr)),
                early_exit: None,
            });

            let _ = state.metrics_tx.try_send(event);

            res
        }
        Err(e) => {
            let status_code = match &e {
                GatewayError::Overloaded => 503,
                GatewayError::ConnectionFailed(_) => 502,
                _ => 500,
            };
            let duration = start.elapsed().as_millis() as u64;
            let _ = state.metrics_tx.try_send(MetricEvent::Request(
                RequestMetric {
                    route: Arc::clone(matched_route_key),
                    status_code,
                    timestamp: Utc::now(),
                    duration_total_ms: duration,
                    duration_upstream_ms,
                    bytes_in,
                    bytes_out: 0,
                    client_ip: ip,
                    method,
                    upstream_addr: Some(Arc::clone(&upstream.pool.server_addr)),
                    early_exit: Some(EarlyExit::UpstreamError),
                },
            ));
            e.into_response()
        }
    }
}
