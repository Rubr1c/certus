use axum::body::Bytes;
use dashmap::DashMap;
use hyper::{HeaderMap, Method, StatusCode};
use moka::sync::Cache;

use crate::server::middleware::cache::{
    CacheKey, CachedResponse, dyn_cache, static_cache,
};

#[test]
fn dyn_cache_miss_on_empty() {
    let cache: Cache<CacheKey, CachedResponse> = Cache::new(100);
    let ck = CacheKey { token: None, path: "/test".to_string() };

    let res = dyn_cache::try_find(&cache, "/test", &ck, &Method::GET);

    assert!(res.is_none());
}

#[test]
fn dyn_cache_hit_on_get() {
    let cache: Cache<CacheKey, CachedResponse> = Cache::new(100);
    let ck = CacheKey { token: None, path: "/test".to_string() };

    let cached = CachedResponse {
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        body: Bytes::from("cached"),
    };
    cache.insert(CacheKey { token: None, path: "/test".to_string() }, cached);

    let res = dyn_cache::try_find(&cache, "/test", &ck, &Method::GET);

    assert!(res.is_some());
    assert_eq!(res.unwrap().status(), StatusCode::OK);
}

#[test]
fn dyn_cache_miss_on_post() {
    let cache: Cache<CacheKey, CachedResponse> = Cache::new(100);
    let ck = CacheKey { token: None, path: "/test".to_string() };

    let cached = CachedResponse {
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        body: Bytes::from("cached"),
    };
    cache.insert(CacheKey { token: None, path: "/test".to_string() }, cached);

    let res = dyn_cache::try_find(&cache, "/test", &ck, &Method::POST);

    assert!(res.is_none());
}

#[test]
fn dyn_cache_different_tokens_are_different_keys() {
    let cache: Cache<CacheKey, CachedResponse> = Cache::new(100);

    let cached = CachedResponse {
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        body: Bytes::from("user1"),
    };
    cache.insert(
        CacheKey {
            token: Some("token_a".to_string()),
            path: "/test".to_string(),
        },
        cached,
    );

    let ck = CacheKey {
        token: Some("token_b".to_string()),
        path: "/test".to_string(),
    };
    let res = dyn_cache::try_find(&cache, "/test", &ck, &Method::GET);

    assert!(res.is_none());
}

#[test]
fn static_cache_miss_on_empty() {
    let cache: DashMap<String, CachedResponse> = DashMap::new();

    let res = static_cache::try_find(&cache, "/missing");

    assert!(res.is_none());
}

#[test]
fn static_cache_hit() {
    let cache: DashMap<String, CachedResponse> = DashMap::new();

    cache.insert(
        "/static".to_string(),
        CachedResponse {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: Bytes::from("static content"),
        },
    );

    let res = static_cache::try_find(&cache, "/static");

    assert!(res.is_some());
    assert_eq!(res.unwrap().status(), StatusCode::OK);
}
