use hyper::header;

use super::server;
use crate::middleware::forwarding;

/// Checks if the health of the server is ok
///
/// # Arguments
///
/// * `upstream` - target server trying to check
#[inline(always)]
pub async fn health_ok(upstream: &server::UpstreamServer) -> bool {
    tracing::debug!(
        upstream = %upstream.pool.server_addr,
        "Running upstream health check"
    );
    let req = match axum::extract::Request::builder()
        .method(hyper::Method::GET)
        .uri("/")
        .header(header::HOST, upstream.pool.hostname.as_str())
        .body(axum::body::Body::empty())
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(
                upstream = %upstream.pool.server_addr,
                err = ?e,
                "Failed to build health request"
            );
            return false;
        }
    };

    match forwarding::handle_request(upstream, req, 2000).await {
        Ok(res) => {
            let status = res.status();
            let healthy = status.is_success() || status.is_redirection();

            if healthy {
                tracing::debug!(
                    upstream = %upstream.pool.server_addr,
                    status_code = status.as_u16(),
                    "Upstream health check passed"
                );
            } else {
                tracing::debug!(
                    upstream = %upstream.pool.server_addr,
                    status_code = status.as_u16(),
                    "Upstream health check returned unhealthy status"
                );
            }

            healthy
        }
        Err(err) => {
            tracing::debug!(
                upstream = %upstream.pool.server_addr,
                err = ?err,
                "Upstream health check request failed"
            );
            false
        }
    }
}
