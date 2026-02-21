pub mod dyn_cache;
pub mod static_cache;

use axum::{
    body::{Body, Bytes},
    response::IntoResponse,
};
use dashmap::DashMap;
use hyper::{HeaderMap, Response, StatusCode};
use moka::sync::Cache;

//TODO: add redis to cache backend and make it configable

pub enum StaticCacheBackend {
    InMemory(DashMap<String, CachedResponse>),
}

impl StaticCacheBackend {
    pub fn get(&self, key: &str) -> Option<CachedResponse> {
        match self {
            StaticCacheBackend::InMemory(map) => {
                map.get(key).map(|v| v.clone())
            }
        }
    }

    pub fn set(&self, key: String, value: CachedResponse) {
        match self {
            StaticCacheBackend::InMemory(map) => {
                map.insert(key, value);
            }
        }
    }
}

pub enum DynCacheBackend {
    InMemory(Cache<CacheKey, CachedResponse>),
}

impl DynCacheBackend {
    pub fn get(&self, key: &CacheKey) -> Option<CachedResponse> {
        match self {
            DynCacheBackend::InMemory(cache) => cache.get(key),
        }
    }

    pub fn set(&self, key: CacheKey, value: CachedResponse) {
        match self {
            DynCacheBackend::InMemory(cache) => cache.insert(key, value),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct CacheKey {
    pub token: Option<String>,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}

impl IntoResponse for CachedResponse {
    fn into_response(self) -> Response<Body> {
        let mut response = Response::new(Body::from(self.body));

        *response.status_mut() = self.status;
        *response.headers_mut() = self.headers;

        response
    }
}
