use axum::{
    body::{Body, to_bytes},
    response::IntoResponse,
};
use hyper::{Method, Response, body::Incoming, header::CONTENT_LENGTH};

use crate::server::middleware::cache::{
    CacheKey, CachedResponse, DynCacheBackend,
};

/// Tries to save a response to a cache
///
/// # Arguments
///
/// * `response` - response stream from the server
/// * `method` - http method used for request
/// * `cache` - target cache to save in
/// * `ck` - CacheKey for the request
/// * `ttl` - optional ttl overide
/// * `max_size` - max response body size (bytes) to cache
#[inline]
pub async fn try_save(
    response: Response<Incoming>,
    method: &Method,
    cache: &DynCacheBackend,
    ck: CacheKey<'_>,
    ttl: Option<u64>,
    max_size: u64,
) -> Response<Body> {
    if method != Method::GET {
        return response.into_response();
    }

    if !response.status().is_success() {
        return response.into_response();
    }

    let content_length = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    if let Some(len) = content_length {
        if len > max_size {
            tracing::debug!("Response too large to cache ({len} bytes)");
            return response.into_response();
        }
    }

    let (parts, body) = response.into_parts();
    let body =
        //TODO: set limit
        //
        //ISSUE: when setting max size then reading the error the
        //       response gets consumed so returning the original
        //       is not possible. a fix that is performant is needed.
        //       Content-Length is checked above, but chunked/streaming
        //       responses without Content-Length can still exceed max_size.
        to_bytes(Body::new(body), usize::MAX).await.unwrap_or_default();

    let cached =
        CachedResponse { status: parts.status, headers: parts.headers, body };

    let response = cached.clone().into_response();

    tracing::info!("Saving to cache");
    match ttl {
        Some(secs) => cache.set_ex(ck, cached, &secs).await,
        _ => cache.set(ck, cached).await,
    }

    response
}

/// Tries to find a response in cache
///
/// # Arguments
///
/// * `state` - gateway app state
/// * `path` - full path of the request
/// * `ck` - CacheKey for the request
#[inline]
pub async fn try_find(
    cache: &DynCacheBackend,
    path: &str,
    ck: &CacheKey<'_>,
) -> Option<Response<Body>> {
    match cache.get(ck).await {
        Some(res) => {
            tracing::info!("Returning cached response to {}", path);
            return Some(res.into_response());
        }
        _ => {
            tracing::info!("Response not found in cache");
            None
        }
    }
}
