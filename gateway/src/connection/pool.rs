use std::sync::atomic::Ordering;

use crate::{
    error::GatewayError, upstream::protocol::PooledConnection,
    upstream::server::UpstreamServer,
};

use super::open::open_connection;

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
    while let Some(sender) = upstream.pool.idle_connections.pop() {
        let alive = match &sender {
            PooledConnection::Http1(s) => s.is_ready(),
            PooledConnection::Http2(s) => s.is_ready(),
        };

        if alive {
            tracing::info!("Found idle connetion");
            upstream.active_connctions.fetch_add(1, Ordering::AcqRel);
            return Ok(sender);
        }

        upstream.pool.total_connections.fetch_sub(1, Ordering::AcqRel);
        tracing::warn!("Discarded dead idle connection");
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

    upstream.pool.total_connections.fetch_add(1, Ordering::AcqRel);
    upstream.active_connctions.fetch_add(1, Ordering::AcqRel);

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
    upstream.active_connctions.fetch_sub(1, Ordering::AcqRel);

    if reusable {
        upstream.pool.idle_connections.push(sender);
    } else {
        tracing::info!("Connetion not reusable");
        upstream.pool.total_connections.fetch_sub(1, Ordering::AcqRel);
    }
}
