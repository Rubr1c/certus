use std::{net::SocketAddr, sync::Arc, time::Instant};

use axum::response::IntoResponse;
use hyper::header;
use tracing::{Level, instrument};

use crate::{
    config::{Config, RouteConfig},
    error::GatewayError,
    metrics::{CacheMetric, MetricEvent, RequestMetric},
    middleware::{rate_limit, request_context::RequestContext},
    schema::ReqResSchemaDTO,
    server::state::{app_state::AppState, routing_table::RoutingTable},
};

use super::{
    auth,
    cache::{self, static_cache},
    forwarding,
    load_balance::selector,
    rate_limit::limiter,
};

#[inline]
fn resolve_route<'a>(
    routing_table: &'a Arc<RoutingTable>,
    config: &'a Arc<Config>,
    path: &str,
) -> Result<(&'a Arc<str>, (&'a String, &'a RouteConfig)), GatewayError> {
    let matched_route_key = match routing_table.router.at(path) {
        Ok(match_result) => match_result.value,
        Err(_) => return Err(GatewayError::NotFound),
    };

    let target_route = config
        .routes
        .get_key_value(matched_route_key.as_ref())
        .expect("route should exist");

    Ok((matched_route_key, target_route))
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
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<SocketAddr>,
    mut req: hyper::Request<axum::body::Body>,
) -> impl IntoResponse {
    let start = Instant::now();

    let config = state.config.load();
    let routing_table = state.routing_table.load();
    let ctx = RequestContext::extract(&req, addr, &config);

    let (matched_route_key, target_route) =
        match resolve_route(&routing_table, &config, &ctx.path) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };

    let span = tracing::span!(Level::INFO, "route");
    let _span_guard = span.enter();

    let headers = req.headers();
    let token_bucket_key =
        rate_limit::key::build(&config.rate_limit, &ctx, headers);

    match limiter::run(
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
                RequestMetric::rate_limited(
                    Arc::clone(matched_route_key),
                    duration,
                    ctx.bytes_in,
                    ctx.ip,
                    ctx.method.clone(),
                ),
            ));
            return err.into_response();
        }
    }

    let ck = cache::key::build(&ctx);
    let c_policy = cache::policy::build(
        target_route.1,
        &ctx.method,
        ctx.token.as_deref(),
        headers,
    );

    if c_policy.lookup {
        //can maybe combine both cache methods into one fn
        if let Some(res) =
            static_cache::try_find(&state.static_cache, &ctx.path).await
        {
            let _ = state.metrics_tx.try_send(MetricEvent::Cache(
                CacheMetric::hit(Arc::clone(matched_route_key)),
            ));

            return res;
        }

        match cache::dynamic::try_find(&state.cache, &ctx.path, &ck).await {
            Some(res) => {
                let _ = state.metrics_tx.try_send(MetricEvent::Cache(
                    CacheMetric::hit(Arc::clone(matched_route_key)),
                ));

                return res;
            }
            _ => {
                let _ = state.metrics_tx.try_send(MetricEvent::Cache(
                    CacheMetric::miss(Arc::clone(matched_route_key)),
                ));
            }
        }
    } else if c_policy.bypass {
        let _ = state.metrics_tx.try_send(MetricEvent::Cache(
            CacheMetric::bypass(Arc::clone(matched_route_key)),
        ));
    }

    let server = selector::run(
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

    match auth::gate::run(
        req.headers_mut(),
        ctx.token.as_deref(),
        &config,
        target_route.1.needs_auth,
    ) {
        Ok(_) => {}
        Err(e) => {
            let duration = start.elapsed().as_millis() as u64;
            let _ = state.metrics_tx.try_send(MetricEvent::Request(
                RequestMetric::unauthorized(
                    Arc::clone(matched_route_key),
                    duration,
                    ctx.bytes_in,
                    ctx.ip,
                    ctx.method.clone(),
                ),
            ));
            return e.into_response();
        }
    }

    forwarding::prepare(target_route.1, &mut req, &upstream);

    let req_headers = req.headers().clone();

    let upstream_start = std::time::Instant::now();

    let res = forwarding::handle_request(
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
                .get(header::CONTENT_LENGTH)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            if c_policy.no_store {
                let _ = state.schema_tx.try_send(ReqResSchemaDTO {
                    full_path: Arc::clone(&ctx.path),
                    method: ctx.method.clone(),
                    query_params: ctx.query.as_ref().map(Arc::clone),
                    status_code,
                    req_headers,
                    res_headers,
                    body_schema: None,
                });

                let duration = start.elapsed().as_millis() as u64;
                let _ = state.metrics_tx.try_send(MetricEvent::Request(
                    RequestMetric::success(
                        Arc::clone(matched_route_key),
                        status_code.as_u16(),
                        duration,
                        duration_upstream_ms,
                        ctx.bytes_in,
                        bytes_out,
                        ctx.ip,
                        ctx.method.clone(),
                        Arc::clone(&upstream.pool.server_addr),
                    ),
                ));

                return response.into_response();
            }
            let (res, body_schema) = cache::dynamic::try_save(
                response,
                &ctx.method,
                &state.cache,
                ck,
                c_policy.max_age,
                config.cache.max_size,
            )
            .await;

            let _ = state.schema_tx.try_send(ReqResSchemaDTO {
                full_path: Arc::clone(&ctx.path),
                method: ctx.method.clone(),
                query_params: ctx.query.as_ref().map(Arc::clone),
                status_code,
                req_headers,
                res_headers,
                body_schema,
            });

            let duration = start.elapsed().as_millis() as u64;
            let _ = state.metrics_tx.try_send(MetricEvent::Request(
                RequestMetric::success(
                    Arc::clone(matched_route_key),
                    res.status().as_u16(),
                    duration,
                    duration_upstream_ms,
                    ctx.bytes_in,
                    bytes_out,
                    ctx.ip,
                    ctx.method.clone(),
                    Arc::clone(&upstream.pool.server_addr),
                ),
            ));

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
                RequestMetric::upstream_error(
                    Arc::clone(matched_route_key),
                    status_code,
                    duration,
                    duration_upstream_ms,
                    ctx.bytes_in,
                    ctx.ip,
                    ctx.method,
                    Arc::clone(&upstream.pool.server_addr),
                ),
            ));
            e.into_response()
        }
    }
}
