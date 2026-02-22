use axum::body::Bytes;
use dashmap::DashMap;
use hyper::{HeaderMap, Method, StatusCode};
use moka::sync::Cache;

use crate::server::middleware::cache::{
    CacheKey, CachedResponse, DynCacheBackend, StaticCacheBackend, dyn_cache,
    static_cache,
};

#[tokio::test]
async fn dyn_cache_miss_on_empty() {
    let cache = DynCacheBackend::InMemory(Cache::new(100));
    let ck = CacheKey { token: None, path: "/test".to_string() };

    let res = dyn_cache::try_find(&cache, "/test", &ck, &Method::GET).await;

    assert!(res.is_none());
}

#[tokio::test]
async fn dyn_cache_hit_on_get() {
    let inner = Cache::new(100);
    let ck = CacheKey { token: None, path: "/test".to_string() };

    let cached = CachedResponse {
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        body: Bytes::from("cached"),
    };
    inner.insert(CacheKey { token: None, path: "/test".to_string() }, cached);
    let cache = DynCacheBackend::InMemory(inner);

    let res = dyn_cache::try_find(&cache, "/test", &ck, &Method::GET).await;

    assert!(res.is_some());
    assert_eq!(res.unwrap().status(), StatusCode::OK);
}

#[tokio::test]
async fn dyn_cache_miss_on_post() {
    let inner = Cache::new(100);
    let ck = CacheKey { token: None, path: "/test".to_string() };

    let cached = CachedResponse {
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        body: Bytes::from("cached"),
    };
    inner.insert(CacheKey { token: None, path: "/test".to_string() }, cached);
    let cache = DynCacheBackend::InMemory(inner);

    let res = dyn_cache::try_find(&cache, "/test", &ck, &Method::POST).await;

    assert!(res.is_none());
}

#[tokio::test]
async fn dyn_cache_different_tokens_are_different_keys() {
    let inner = Cache::new(100);

    let cached = CachedResponse {
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        body: Bytes::from("user1"),
    };
    inner.insert(
        CacheKey {
            token: Some("token_a".to_string()),
            path: "/test".to_string(),
        },
        cached,
    );
    let cache = DynCacheBackend::InMemory(inner);

    let ck = CacheKey {
        token: Some("token_b".to_string()),
        path: "/test".to_string(),
    };
    let res = dyn_cache::try_find(&cache, "/test", &ck, &Method::GET).await;

    assert!(res.is_none());
}

#[test]
fn static_cache_miss_on_empty() {
    let cache = StaticCacheBackend::InMemory(DashMap::new());

    let res = static_cache::try_find(&cache, "/missing");

    assert!(res.is_none());
}

#[test]
fn static_cache_hit() {
    let inner = DashMap::new();
    inner.insert(
        "/static".to_string(),
        CachedResponse {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: Bytes::from("static content"),
        },
    );
    let cache = StaticCacheBackend::InMemory(inner);

    let res = static_cache::try_find(&cache, "/static");

    assert!(res.is_some());
    assert_eq!(res.unwrap().status(), StatusCode::OK);
}
