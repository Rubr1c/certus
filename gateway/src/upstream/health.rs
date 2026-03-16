use axum::{body::Body, extract::Request};
use hyper::{Method, header};

use super::server::UpstreamServer;
use crate::middleware::forwarding;

/// Checks if the health of the server is ok
///
/// # Arguments
///
/// * `upstream` - target server trying to check
pub async fn health_ok(upstream: &UpstreamServer) -> bool {
    let req = match Request::builder()
        .method(Method::GET)
        .uri("/")
        .header(header::HOST, upstream.pool.hostname.as_str())
        .body(Body::empty())
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(err = ?e, "error building health req");
            return false;
        }
    };

    match forwarding::handle_request(&upstream, req, 2000).await {
        Ok(res) => {
            let status = res.status();
            status.is_success() || status.is_redirection()
        }
        Err(_) => false,
    }
}
