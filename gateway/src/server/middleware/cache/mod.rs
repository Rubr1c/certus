pub mod dyn_cache;
pub mod static_cache;

use axum::{
    body::{Body, Bytes},
    response::IntoResponse,
};
use bb8_redis::RedisConnectionManager;
use dashmap::DashMap;
use hyper::{HeaderMap, Response, StatusCode, header::HeaderName};
use moka::sync::Cache;
use redis::AsyncTypedCommands;
use serde::{Deserialize, Serialize};

/// Cache backend for static routes (keyed by path string).
/// Only in-memory for now since static responses rarely change.
pub enum StaticCacheBackend {
    InMemory(DashMap<String, CachedResponse>),
    Redis(bb8::Pool<RedisConnectionManager>),
}

impl StaticCacheBackend {
    pub async fn get(&self, key: &str) -> Option<CachedResponse> {
        match self {
            StaticCacheBackend::InMemory(map) => {
                map.get(key).map(|v| v.clone())
            }
            StaticCacheBackend::Redis(pool) => {
                let mut conn = pool.get().await.ok()?;
                let json: Option<String> = conn.get(&key).await.ok()?;
                let json = json?;
                let s: SerializableCachedResponse =
                    serde_json::from_str(&json).ok()?;
                Some(s.into())
            }
        }
    }

    pub async fn set(&self, key: String, value: CachedResponse) {
        match self {
            StaticCacheBackend::InMemory(map) => {
                map.insert(key, value);
            }
            StaticCacheBackend::Redis(pool) => {
                let Ok(mut conn) = pool.get().await else { return };
                let Ok(json) = serde_json::to_string(
                    &SerializableCachedResponse::from(&value),
                ) else {
                    return;
                };
                let _: Result<(), _> = conn.set(&key, json).await;
            }
        }
    }
}

/// Cache backend for dynamic routes (keyed by path + optional auth token).
/// Supports in-memory (moka) or Redis (bb8 connection pool).
///
/// Redis serialization: CachedResponse can't implement redis traits directly
/// (redis v1.0.4's `get` always returns `Option<String>`, and `set` requires
/// `ToSingleRedisArg`). Instead we convert through `SerializableCachedResponse`
/// to/from a JSON string, which works natively with the typed redis commands.
pub enum DynCacheBackend {
    InMemory(Cache<CacheKey, CachedResponse>),
    Redis { pool: bb8::Pool<RedisConnectionManager>, ttl: Option<u64> },
}

impl DynCacheBackend {
    pub async fn get(&self, key: &CacheKey) -> Option<CachedResponse> {
        match self {
            DynCacheBackend::InMemory(cache) => cache.get(key),
            DynCacheBackend::Redis { pool, .. } => {
                let mut conn = pool.get().await.ok()?;
                let redis_key = format!(
                    "cache:{}:{}",
                    key.path,
                    key.token.as_deref().unwrap_or("")
                );
                let json: Option<String> = conn.get(&redis_key).await.ok()?;
                let json = json?;
                let s: SerializableCachedResponse =
                    serde_json::from_str(&json).ok()?;
                Some(s.into())
            }
        }
    }

    pub async fn set(&self, key: CacheKey, value: CachedResponse) {
        match self {
            DynCacheBackend::InMemory(cache) => cache.insert(key, value),
            DynCacheBackend::Redis { pool, ttl } => {
                let Ok(mut conn) = pool.get().await else { return };
                let redis_key = format!(
                    "cache:{}:{}",
                    key.path,
                    key.token.as_deref().unwrap_or("")
                );
                let Ok(json) = serde_json::to_string(
                    &SerializableCachedResponse::from(&value),
                ) else {
                    return;
                };
                match ttl {
                    Some(secs) => {
                        let _: Result<(), _> =
                            conn.set_ex(&redis_key, json, *secs).await;
                    }
                    None => {
                        let _: Result<(), _> = conn.set(&redis_key, json).await;
                    }
                }
            }
        }
    }

    pub async fn set_ex(
        &self,
        key: CacheKey,
        value: CachedResponse,
        ttl: &u64,
    ) {
        match self {
            DynCacheBackend::InMemory(cache) => cache.insert(key, value),
            DynCacheBackend::Redis { pool, ttl: _ } => {
                let Ok(mut conn) = pool.get().await else { return };
                let redis_key = format!(
                    "cache:{}:{}",
                    key.path,
                    key.token.as_deref().unwrap_or("")
                );
                let Ok(json) = serde_json::to_string(
                    &SerializableCachedResponse::from(&value),
                ) else {
                    return;
                };

                let _: Result<(), _> =
                    conn.set_ex(&redis_key, json, *ttl).await;
            }
        }
    }
}

/// Composite key for dynamic cache entries.
/// Different auth tokens get separate cache entries for the same path.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize)]
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

/// Serde-friendly intermediate representation of CachedResponse.
/// CachedResponse itself uses hyper types (StatusCode, HeaderMap, Bytes)
/// which don't implement Serialize/Deserialize, so we convert through this
/// struct for Redis JSON storage.
#[derive(Serialize, Deserialize)]
struct SerializableCachedResponse {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl From<&CachedResponse> for SerializableCachedResponse {
    fn from(r: &CachedResponse) -> Self {
        Self {
            status: r.status.as_u16(),
            headers: r
                .headers
                .iter()
                .map(|(k, v)| {
                    (
                        k.as_str().to_string(),
                        v.to_str().unwrap_or("").to_string(),
                    )
                })
                .collect(),
            body: r.body.to_vec(),
        }
    }
}

impl From<SerializableCachedResponse> for CachedResponse {
    fn from(s: SerializableCachedResponse) -> Self {
        let mut headers = HeaderMap::new();
        for (k, v) in s.headers {
            if let (Ok(name), Ok(val)) = (
                k.parse::<HeaderName>(),
                v.parse::<hyper::header::HeaderValue>(),
            ) {
                headers.insert(name, val);
            }
        }
        Self {
            status: StatusCode::from_u16(s.status).unwrap_or(StatusCode::OK),
            headers,
            body: Bytes::from(s.body),
        }
    }
}

impl IntoResponse for CachedResponse {
    fn into_response(self) -> Response<Body> {
        let mut response = Response::new(Body::from(self.body));

        *response.status_mut() = self.status;
        *response.headers_mut() = self.headers;

        response
    }
}
