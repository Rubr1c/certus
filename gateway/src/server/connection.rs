use std::{sync::atomic::Ordering, time::Duration};

use axum::body::Body;
use hyper::{Method, Request, client::conn, header};
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpStream;
use tracing::instrument;

use crate::server::{
    error::GatewayError,
    middleware::handler,
    upstream::{PooledConnection, Protocol, UpstreamServer},
};

//TODO: make sure atomic ordering correct

/// Opens a connection to a tcp stream
///
/// # Arguments
///
/// * `upstream` - target server to conenct to
/// * `timeout` - when to timeout trying to connect
///
/// # Errors
///
/// Returns an error if:
/// * Failed to connect to server
/// * Timed out while trying to connect
/// * Failed to perform handshake with server
#[instrument(skip_all, fields(protocol = ?upstream.pool.protocol))]
pub async fn open_connection(
    upstream: &UpstreamServer,
    timeout: u64,
) -> Result<PooledConnection, Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Connecting to new upstream");
    let connect_future = TcpStream::connect(upstream.pool.server_addr);
    let stream =
        tokio::time::timeout(Duration::from_secs(timeout), connect_future)
            .await??;

    let io = TokioIo::new(stream);

    let sender = match upstream.pool.protocol {
        Protocol::HTTP1 => {
            let (sender, conn) = conn::http1::handshake::<_, Body>(io).await?;
            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    tracing::error!(?err, "Connection failed");
                }
            });
            PooledConnection::Http1(sender)
        }
        Protocol::HTTP2 => {
            let exec = TokioExecutor::new();
            let (sender, conn) = conn::http2::handshake(exec, io).await?;
            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    tracing::error!(?err, "Connection failed");
                }
            });
            PooledConnection::Http2(sender)
        }
    };

    tracing::info!("Connetion made");

    Ok(sender)
}

/// Tries to borrow an idle connection if not able it
/// opens a new one
///
/// # Arguments
///
/// * `upstream` - target server to conenct to
/// * `timeout` - when to timeout trying to make a new connection
///
/// # Errors
///
/// Returns an error if:
/// * Max connections to server reached
/// * Failed to open new connection
pub async fn borrow_connection(
    upstream: &UpstreamServer,
    timeout: u64,
) -> Result<PooledConnection, GatewayError> {
    tracing::info!("Checking for idle connetions");
    if let Some(sender) = upstream.pool.idle_connections.pop() {
        tracing::info!("Found idle connetion");
        upstream.active_connctions.fetch_add(1, Ordering::Release);
        return Ok(sender);
    }
    tracing::info!("No idle connetions found");

    let total = upstream.pool.total_connections.load(Ordering::Acquire);
    if total >= upstream.pool.max_connections {
        tracing::info!(total_cons = total, "Max connetions reached");
        return Err(GatewayError::Overloaded);
    }

    let sender = open_connection(upstream, timeout)
        .await
        .map_err(|e| GatewayError::ConnectionFailed(e.to_string()))?;

    upstream.pool.total_connections.fetch_add(1, Ordering::Release);
    upstream.active_connctions.fetch_add(1, Ordering::Release);

    Ok(sender)
}

/// Removes connection to server and keeps it idle if possible
///
/// # Arguments
///
/// * `upstream` - target server to release
/// * `sender` - connection to the server
/// * `reusable` - if the connection can be reused and put in idle
pub async fn release_connection(
    upstream: &UpstreamServer,
    sender: PooledConnection,
    reusable: bool,
) {
    tracing::info!("Releasing connetion");
    upstream.active_connctions.fetch_sub(1, Ordering::Release);

    if reusable {
        upstream.pool.idle_connections.push(sender);
    } else {
        tracing::info!("Connetion not reusable");
        upstream.pool.total_connections.fetch_sub(1, Ordering::Release);
    }
}

/// Checks if the health of the server is ok
///
/// # Arguments
///
/// * `upstream` - target server trying to check
pub async fn health_ok(upstream: &UpstreamServer) -> bool {
    let req = match Request::builder()
        .method(Method::GET)
        .uri("/health")
        .header(header::HOST, upstream.pool.server_addr.to_string())
        .body(Body::empty())
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(err = ?e, "error building health req");
            return false;
        }
    };

    match handler::handle_request(&upstream, req, 2000).await {
        Ok(res) => {
            if res.status().is_success() {
                return true;
            }
            false
        }
        Err(_) => false,
    }
}
