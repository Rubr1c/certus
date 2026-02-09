use axum::{response::IntoResponse, body::{Body, to_bytes}};
use hyper::{Method, Response, body::Incoming};

use crate::server::{app_state::AppState, middleware::cache::models::{CacheKey, CachedResponse}};


#[inline]
pub async fn try_save(
    response: Response<Incoming>,
    method: &Method,
    state: &AppState,
    ck: CacheKey,
) -> Response<Body> {
    
    let (parts, body) = response.into_parts();
    let body =
        //TODO: set limit
        to_bytes(Body::new(body), usize::MAX).await.unwrap_or_default();

    let cached = CachedResponse {
        status: parts.status,
        headers: parts.headers,
        body,
    };

    let response = cached.clone().into_response();
    // only cache get requests
    if method == Method::GET {
        state.cache.insert(ck, cached);
    }
    response
}


#[inline]
pub fn try_find(
    state: &AppState, 
    path: &str, 
    ck: &CacheKey, 
    method: &Method
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
