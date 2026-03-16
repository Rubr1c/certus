use std::borrow::Cow;

use axum::body::Bytes;
use dashmap::DashMap;
use hyper::{HeaderMap, StatusCode};
use moka::sync::Cache;

use crate::middleware::cache::{
    backend::{DynCacheBackend, StaticCacheBackend},
    dynamic,
    key::CacheKey,
    response::CachedResponse,
    static_cache,
};

#[tokio::test]
async fn dyn_cache_miss_on_empty() {
    let cache = DynCacheBackend::InMemory(Cache::new(100));
    let ck = CacheKey { token: None, path: Cow::Borrowed("/test") };

    let res = dynamic::try_find(&cache, "/test", &ck).await;

    assert!(res.is_none());
}

#[tokio::test]
async fn dyn_cache_hit_on_get() {
    let inner = Cache::new(100);
    let ck = CacheKey { token: None, path: Cow::Borrowed("/test") };

    let cached = CachedResponse {
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        body: Bytes::from("cached"),
    };
    inner.insert(
        CacheKey { token: None, path: Cow::Owned("/test".to_string()) },
        cached,
    );
    let cache = DynCacheBackend::InMemory(inner);

    let res = dynamic::try_find(&cache, "/test", &ck).await;

    assert!(res.is_some());
    assert_eq!(res.unwrap().status(), StatusCode::OK);
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
            token: Some(Cow::Owned("token_a".to_string())),
            path: Cow::Owned("/test".to_string()),
        },
        cached,
    );
    let cache = DynCacheBackend::InMemory(inner);

    let ck = CacheKey {
        token: Some(Cow::Borrowed("token_b")),
        path: Cow::Borrowed("/test"),
    };
    let res = dynamic::try_find(&cache, "/test", &ck).await;

    assert!(res.is_none());
}

#[tokio::test]
async fn static_cache_miss_on_empty() {
    let cache = StaticCacheBackend::InMemory(DashMap::new());

    let res = static_cache::try_find(&cache, "/missing").await;

    assert!(res.is_none());
}

#[tokio::test]
async fn static_cache_hit() {
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

    let res = static_cache::try_find(&cache, "/static").await;

    assert!(res.is_some());
    assert_eq!(res.unwrap().status(), StatusCode::OK);
}
