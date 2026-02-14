use axum::{
    body::{Body, to_bytes},
    response::IntoResponse,
};
use hyper::{Method, Response, body::Incoming};
use moka::sync::Cache;

use crate::server::{
    app_state::AppState,
    middleware::cache::models::{CacheKey, CachedResponse},
};

#[inline]
pub async fn try_save(
    response: Response<Incoming>,
    method: &Method,
    cache: &Cache<CacheKey, CachedResponse>,
    ck: CacheKey,
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
    cache.insert(ck, cached);

    response
}

#[inline]
pub fn try_find(
    state: &AppState,
    path: &str,
    ck: &CacheKey,
    method: &Method,
) -> Option<Response<Body>> {
    match state.cache.get(ck) {
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
