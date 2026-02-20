use axum::{
    body::{Body, to_bytes},
    http,
    response::IntoResponse,
};
use dashmap::DashMap;
use hyper::{Request, Response};

use crate::server::{
    middleware::{cache::CachedResponse, handler},
    upstream::{HttpVersion, UpstreamServer},
};

/// Tries to find a response in static cache
///
/// # Arguments
///
/// * `cache` - static cache to search in
/// * `path` - full path of the request
#[inline]
pub fn try_find(
    cache: &DashMap<String, CachedResponse>,
    path: &str,
) -> Option<Response<Body>> {
    match cache.get(path) {
        Some(res) => {
            tracing::info!("Returning static cached response to {}", path);
            Some(res.clone().into_response())
        }
        None => {
            tracing::debug!("Path {:?} not found in static cache", path);
            None
        }
    }
}

/// Sends a request to an endpoint and saves it in the static cache
///
/// # Arguments
///
/// * `cache` - cache map to save in
/// * `upstream` - server to send request to
/// * `path` - full path of the request
/// * `timeout` - when to timeout trying to connect to server
pub async fn send_and_save(
    cache: &DashMap<String, CachedResponse>,
    upstream: &UpstreamServer,
    path: &String,
    timeout: u64,
) {
    let version = match upstream.pool.http_version {
        HttpVersion::HTTP1 => http::Version::HTTP_11,
        HttpVersion::HTTP2 => http::Version::HTTP_2,
    };

    let full_uri = format!("http://{}{}", upstream.pool.server_addr, path);

    let req = match Request::builder()
        .method(http::Method::GET)
        .uri(full_uri)
        .version(version)
        .header(http::header::HOST, upstream.pool.server_addr.to_string())
        .body(Body::empty())
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("error building static req for {}: {}", path, e);
            return;
        }
    };

    let res = handler::handle_request(&upstream, req, timeout).await;

    //TODO: remove reused code
    match res {
        Ok(response) => {
            let (parts, body) = response.into_parts();

            if !parts.status.is_success() {
                tracing::warn!(
                    "Static path {} returned non-success status: {}",
                    path,
                    parts.status
                );
                return;
            }

            let body =
                to_bytes(Body::new(body), usize::MAX).await.unwrap_or_default();

            let cached = CachedResponse {
                status: parts.status,
                headers: parts.headers,
                body,
            };

            cache.insert(path.clone(), cached);
            tracing::info!("Saved static path {} to cache", path);
        }
        Err(e) => {
            tracing::error!("Failed to fetch static path {}: {}", path, e)
        }
    }
}
