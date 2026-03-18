use std::sync::atomic::Ordering;

use crate::{
    error::GatewayError,
    upstream::{protocol, server},
};

use super::open;

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
#[inline(always)]
pub async fn borrow_connection(
    upstream: &server::UpstreamServer,
    timeout: u64,
) -> Result<protocol::PooledConnection, GatewayError> {
    tracing::debug!("Checking for idle connetions");
    while let Some(sender) = upstream.pool.idle_connections.pop() {
        let alive = match &sender {
            protocol::PooledConnection::Http1(s) => s.is_ready(),
            protocol::PooledConnection::Http2(s) => s.is_ready(),
        };

        if alive {
            tracing::debug!("Found idle connetion");
            upstream.active_connctions.fetch_add(1, Ordering::AcqRel);
            return Ok(sender);
        }

        upstream.pool.total_connections.fetch_sub(1, Ordering::AcqRel);
        tracing::warn!("Discarded dead idle connection");
    }

    tracing::debug!("No idle connetions found");

    let total = upstream.pool.total_connections.load(Ordering::Acquire);
    if total >= upstream.pool.max_connections {
        tracing::warn!(total_cons = total, "Max connetions reached");
        return Err(GatewayError::Overloaded);
    }

    let sender = open::open_connection(upstream, timeout)
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
#[inline(always)]
pub async fn release_connection(
    upstream: &server::UpstreamServer,
    sender: protocol::PooledConnection,
    reusable: bool,
) {
    tracing::debug!("Releasing connetion");
    upstream.active_connctions.fetch_sub(1, Ordering::AcqRel);

    if reusable {
        upstream.pool.idle_connections.push(sender);
    } else {
        tracing::debug!("Connetion not reusable");
        upstream.pool.total_connections.fetch_sub(1, Ordering::AcqRel);
    }
}
