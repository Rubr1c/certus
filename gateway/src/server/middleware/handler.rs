use std::sync::atomic::Ordering;

use axum::{body::Body, extract::Request, response::Response};
use hyper::body::Incoming;
use tracing::instrument;

use crate::server::{
    connection,
    error::GatewayError,
    upstream::{HealthState, PooledConnection, UpstreamServer},
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
async fn forward_request(
    conn: PooledConnection,
    req: Request<Body>,
) -> Result<(Response<Incoming>, PooledConnection), GatewayError> {
    tracing::info!("Atempting forward request");
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

    tracing::info!("Request forwarded successfuly");

    Ok((res, sender))
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
#[instrument(name = "request", skip_all, fields(server = %upstream.pool.server_addr))]
pub async fn handle_request(
    upstream: &UpstreamServer,
    req: Request<Body>,
    timeout: u64,
) -> Result<Response<Incoming>, GatewayError> {
    let sender = connection::borrow_connection(&upstream, timeout).await?;

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

    let reusable =
        !res.headers().get("connection").is_some_and(|v| v == "close");

    connection::release_connection(&upstream, sender, reusable).await;

    Ok(res)
}
