use axum::{
    body::{Body, to_bytes},
    response::IntoResponse,
};
use hyper::{Method, Response, body::Incoming};

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
#[inline]
pub async fn try_save(
    response: Response<Incoming>,
    method: &Method,
    cache: &DynCacheBackend,
    ck: CacheKey,
    ttl: Option<u64>,
) -> Response<Body> {
    if method != Method::GET {
        return response.into_response();
    }

    let (parts, body) = response.into_parts();
    let body =
        //TODO: set limit
        //
        //ISSUE: when setting max size then reading the error the
        //       response gets consumed so returning the original
        //       is not possible. a fix that is performant is needed.  
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
/// * `method` - http method used for request
#[inline]
pub async fn try_find(
    cache: &DynCacheBackend,
    path: &str,
    ck: &CacheKey,
    method: &Method,
) -> Option<Response<Body>> {
    match cache.get(ck).await {
        Some(res) => {
            if method == Method::GET {
                tracing::info!("Returning cached response to {}", path);
                return Some(res.into_response());
            }
            None
        }
        _ => {
            tracing::info!("Response not found in cache");
            None
        }
    }
}
