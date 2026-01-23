use axum::{
    body::{Body, to_bytes},
    http,
};
use dashmap::DashMap;
use hyper::Request;

use crate::server::{
    middleware::{cache::models::CachedResponse, handler},
    upstream::models::UpstreamServer,
};

pub async fn send_and_save(
    cache: &DashMap<String, CachedResponse>,
    upstream: &UpstreamServer,
    path: &String,
) {
    let req = match Request::builder()
        .method(http::Method::GET)
        .uri(path)
        .header(http::header::HOST, upstream.pool.server_addr.to_string())
        .body(Body::empty())
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("error building static req for {}: {}", path, e);
            return;
        }
    };

    let res = handler::handle_request(&upstream, req).await;

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
