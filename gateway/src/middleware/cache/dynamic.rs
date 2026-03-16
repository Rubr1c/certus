use axum::{body::to_bytes, response::IntoResponse};
use hyper::{
    body::Incoming,
    header::{CONTENT_LENGTH, CONTENT_TYPE},
};

use crate::middleware::cache;
use crate::schema::extractor;

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
    response: hyper::Response<Incoming>,
    method: &hyper::Method,
    cache: &cache::backend::DynBackend,
    ck: cache::key::CacheKey<'_>,
    ttl: Option<u64>,
    max_size: u64,
) -> (hyper::Response<axum::body::Body>, Option<String>) {
    if method != hyper::Method::GET {
        return (response.into_response(), None);
    }

    if !response.status().is_success() {
        return (response.into_response(), None);
    }

    let content_length = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    if let Some(len) = content_length {
        if len > max_size {
            tracing::debug!("Response too large to cache ({len} bytes)");
            return (response.into_response(), None);
        }
    }

    let is_json = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("application/json"))
        .unwrap_or(false);

    let (parts, body) = response.into_parts();
    let body =
        //TODO: set limit
        //
        //ISSUE: when setting max size then reading the error the
        //       response gets consumed so returning the original
        //       is not possible. a fix that is performant is needed.
        //       Content-Length is checked above, but chunked/streaming
        //       responses without Content-Length can still exceed max_size.
        to_bytes(axum::body::Body::new(body), usize::MAX)
            .await
            .unwrap_or_default();

    let body_schema =
        if is_json { extractor::extract_body_schema(&body) } else { None };

    let cached = cache::response::CachedResponse {
        status: parts.status,
        headers: parts.headers,
        body,
    };

    let response = cached.clone().into_response();

    tracing::info!("Saving to cache");
    match ttl {
        Some(secs) => cache.set_ex(ck, cached, &secs).await,
        _ => cache.set(ck, cached).await,
    }

    (response, body_schema)
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
    cache: &cache::backend::DynBackend,
    path: &str,
    ck: &cache::key::CacheKey<'_>,
) -> Option<hyper::Response<axum::body::Body>> {
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
