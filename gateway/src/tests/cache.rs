use std::borrow::Cow;

use dashmap::DashMap;
use moka::sync::Cache;

use crate::middleware::cache;

#[tokio::test]
async fn dyn_cache_miss_on_empty() {
    let cache = cache::backend::DynBackend::InMemory(Cache::new(100));
    let ck = cache::key::CacheKey { token: None, path: Cow::Borrowed("/test") };

    let res = cache::dynamic::try_find(&cache, "/test", &ck).await;

    assert!(res.is_none());
}

#[tokio::test]
async fn dyn_cache_hit_on_get() {
    let inner = Cache::new(100);
    let ck = cache::key::CacheKey { token: None, path: Cow::Borrowed("/test") };

    let cached = cache::response::CachedResponse {
        status: hyper::StatusCode::OK,
        headers: hyper::HeaderMap::new(),
        body: axum::body::Bytes::from("cached"),
    };
    inner.insert(
        cache::key::CacheKey {
            token: None,
            path: Cow::Owned("/test".to_string()),
        },
        cached,
    );
    let cache = cache::backend::DynBackend::InMemory(inner);

    let res = cache::dynamic::try_find(&cache, "/test", &ck).await;

    assert!(res.is_some());
    assert_eq!(res.unwrap().status(), hyper::StatusCode::OK);
}

#[tokio::test]
async fn dyn_cache_different_tokens_are_different_keys() {
    let inner = Cache::new(100);

    let cached = cache::response::CachedResponse {
        status: hyper::StatusCode::OK,
        headers: hyper::HeaderMap::new(),
        body: axum::body::Bytes::from("user1"),
    };
    inner.insert(
        cache::key::CacheKey {
            token: Some(Cow::Owned("token_a".to_string())),
            path: Cow::Owned("/test".to_string()),
        },
        cached,
    );
    let cache = cache::backend::DynBackend::InMemory(inner);

    let ck = cache::key::CacheKey {
        token: Some(Cow::Borrowed("token_b")),
        path: Cow::Borrowed("/test"),
    };
    let res = cache::dynamic::try_find(&cache, "/test", &ck).await;

    assert!(res.is_none());
}

#[tokio::test]
async fn static_cache_miss_on_empty() {
    let cache = cache::backend::StaticBackend::InMemory(DashMap::new());

    let res = cache::static_cache::try_find(&cache, "/missing").await;

    assert!(res.is_none());
}

#[tokio::test]
async fn static_cache_hit() {
    let inner = DashMap::new();
    inner.insert(
        "/static".to_string(),
        cache::response::CachedResponse {
            status: hyper::StatusCode::OK,
            headers: hyper::HeaderMap::new(),
            body: axum::body::Bytes::from("static content"),
        },
    );
    let cache = cache::backend::StaticBackend::InMemory(inner);

    let res = cache::static_cache::try_find(&cache, "/static").await;

    assert!(res.is_some());
    assert_eq!(res.unwrap().status(), hyper::StatusCode::OK);
}
