use std::sync::atomic::Ordering;

use hyper::{body::Incoming, header};
use tracing::instrument;

use crate::{
    config::RouteConfig,
    connection::pool,
    error::GatewayError,
    upstream::{
        protocol::{HttpVersion, PooledConnection},
        server::{HealthState, UpstreamServer},
    },
};

/// Takes a connection to a server then forwards a request
///
/// # Arguments
///
/// * `conn` - connection to the server
/// * `req` - request sent from client
///
/// # Errors
///
/// Returns an error if:
/// * Failed to send request to server
#[inline(always)]
async fn forward_request(
    conn: PooledConnection,
    req: axum::extract::Request,
) -> Result<(hyper::Response<Incoming>, PooledConnection), GatewayError> {
    tracing::debug!("Atempting forward request");
    let (res, sender) =
        match conn {
            PooledConnection::Http1(mut sender) => {
                let res = sender.send_request(req).await.map_err(|e| {
                    GatewayError::ConnectionFailed(e.to_string())
                })?;
                (res, PooledConnection::Http1(sender))
            }
            PooledConnection::Http2(mut sender) => {
                let res = sender.send_request(req).await.map_err(|e| {
                    GatewayError::ConnectionFailed(e.to_string())
                })?;
                (res, PooledConnection::Http2(sender))
            }
        };

    tracing::debug!("Request forwarded successfuly");

    Ok((res, sender))
}

#[inline(always)]
pub fn prepare(
    config: &RouteConfig,
    req: &mut hyper::Request<axum::body::Body>,
    upstream: &UpstreamServer,
) {
    if matches!(config.http_version, HttpVersion::HTTP1) {
        let pq =
            req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
        *req.uri_mut() = pq.parse().expect("valid path_and_query");

        if !req.headers().contains_key(header::HOST) {
            req.headers_mut().insert(
                header::HOST,
                upstream
                    .pool
                    .hostname
                    .parse()
                    .expect("upstream hostname is valid header value"),
            );
        }
    }
}

/// Takes a server gets a connection to it and forwards it
/// then releases it if needed
///
/// # Arguments
///
/// * `upstream` - target server
/// * `req` - request from the client
/// * `timeout` - when to timeout trying to connect
///
/// # Errors
///
/// Returns an error if:
/// * Failed to get or create a connection
/// * Failed to forward the request
#[inline(always)]
#[instrument(name = "request", skip_all, fields(server = %upstream.pool.server_addr))]
pub async fn handle_request(
    upstream: &UpstreamServer,
    req: axum::extract::Request,
    timeout: u64,
) -> Result<hyper::Response<Incoming>, GatewayError> {
    let sender = pool::borrow_connection(upstream, timeout).await?;

    let (res, sender) = match forward_request(sender, req).await {
        Ok((res, sender)) => (res, sender),
        Err(e) => {
            tracing::error!(err = ?e, "Failed to forward request");
            upstream
                .health_state
                .store(HealthState::Dead as u8, Ordering::Release);
            upstream.active_connctions.fetch_sub(1, Ordering::AcqRel);
            upstream.pool.total_connections.fetch_sub(1, Ordering::AcqRel);
            return Err(e);
        }
    };

    let reusable = res.headers().get("connection").is_none_or(|v| v != "close");

    pool::release_connection(upstream, sender, reusable).await;

    Ok(res)
}
