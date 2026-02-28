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
                read_cached_response_hash(&mut conn, key).await
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
                let _ =
                    write_cached_response_hash(&mut conn, &key, &value).await;
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
                read_cached_response_hash(&mut conn, &redis_key).await
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
                if write_cached_response_hash(&mut conn, &redis_key, &value)
                    .await
                    .is_err()
                {
                    return;
                }
                match ttl {
                    Some(secs) => {
                        let _: Result<(), _> = redis::cmd("EXPIRE")
                            .arg(&redis_key)
                            .arg(*secs)
                            .query_async(&mut *conn)
                            .await;
                    }
                    None => {}
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
                if write_cached_response_hash(&mut conn, &redis_key, &value)
                    .await
                    .is_err()
                {
                    return;
                }

                let _: Result<(), _> = redis::cmd("EXPIRE")
                    .arg(&redis_key)
                    .arg(*ttl)
                    .query_async(&mut *conn)
                    .await;
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

#[derive(Serialize, Deserialize)]
struct SerializableHeaders(Vec<(String, String)>);

fn headers_to_json(headers: &HeaderMap) -> Option<String> {
    let data = SerializableHeaders(
        headers
            .iter()
            .map(|(k, v)| {
                (k.as_str().to_string(), v.to_str().unwrap_or("").to_string())
            })
            .collect(),
    );
    serde_json::to_string(&data).ok()
}

fn headers_from_json(json: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let Ok(data) = serde_json::from_str::<SerializableHeaders>(json) else {
        return headers;
    };
    for (k, v) in data.0 {
        if let (Ok(name), Ok(val)) =
            (k.parse::<HeaderName>(), v.parse::<hyper::header::HeaderValue>())
        {
            headers.insert(name, val);
        }
    }
    headers
}

async fn write_cached_response_hash(
    conn: &mut bb8::PooledConnection<'_, RedisConnectionManager>,
    redis_key: &str,
    value: &CachedResponse,
) -> Result<(), ()> {
    let Some(headers_json) = headers_to_json(&value.headers) else {
        return Err(());
    };
    redis::cmd("HSET")
        .arg(redis_key)
        .arg("status")
        .arg(value.status.as_u16())
        .arg("headers")
        .arg(headers_json)
        .arg("body")
        .arg(value.body.as_ref())
        .query_async(&mut **conn)
        .await
        .map_err(|_| ())
}

async fn read_cached_response_hash(
    conn: &mut bb8::PooledConnection<'_, RedisConnectionManager>,
    redis_key: &str,
) -> Option<CachedResponse> {
    let (status, headers, body): (
        Option<u16>,
        Option<String>,
        Option<Vec<u8>>,
    ) = redis::cmd("HMGET")
        .arg(redis_key)
        .arg("status")
        .arg("headers")
        .arg("body")
        .query_async(&mut **conn)
        .await
        .ok()?;

    let status = status?;
    let headers = headers?;
    let body = body?;

    Some(CachedResponse {
        status: StatusCode::from_u16(status).ok()?,
        headers: headers_from_json(&headers),
        body: Bytes::from(body),
    })
}

impl IntoResponse for CachedResponse {
    fn into_response(self) -> Response<Body> {
        let mut response = Response::new(Body::from(self.body));

        *response.status_mut() = self.status;
        *response.headers_mut() = self.headers;

        response
    }
}
