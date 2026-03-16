use bb8_redis::RedisConnectionManager;
use dashmap::DashMap;
use moka::sync::Cache;

use super::{
    key::{CacheKey, OwnedCacheKey},
    response::CachedResponse,
    serialization,
};

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
                serialization::read_cached_response_hash(&mut conn, key).await
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
                let _ = serialization::write_cached_response_hash(
                    &mut conn, &key, &value,
                )
                .await;
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
    InMemory(Cache<OwnedCacheKey, CachedResponse>),
    Redis { pool: bb8::Pool<RedisConnectionManager>, ttl: Option<u64> },
}

impl DynCacheBackend {
    pub async fn get(&self, key: &CacheKey<'_>) -> Option<CachedResponse> {
        match self {
            DynCacheBackend::InMemory(cache) => {
                cache.get(&key.clone().into_owned())
            }
            DynCacheBackend::Redis { pool, .. } => {
                let mut conn = pool.get().await.ok()?;
                let redis_key = format!(
                    "cache:{}:{}",
                    key.path,
                    key.token.as_deref().unwrap_or("")
                );
                serialization::read_cached_response_hash(&mut conn, &redis_key)
                    .await
            }
        }
    }

    pub async fn set(&self, key: CacheKey<'_>, value: CachedResponse) {
        match self {
            DynCacheBackend::InMemory(cache) => {
                cache.insert(key.into_owned(), value)
            }
            DynCacheBackend::Redis { pool, ttl } => {
                let Ok(mut conn) = pool.get().await else { return };
                let redis_key = format!(
                    "cache:{}:{}",
                    key.path,
                    key.token.as_deref().unwrap_or("")
                );
                if serialization::write_cached_response_hash(
                    &mut conn, &redis_key, &value,
                )
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
        key: CacheKey<'_>,
        value: CachedResponse,
        ttl: &u64,
    ) {
        match self {
            DynCacheBackend::InMemory(cache) => {
                cache.insert(key.into_owned(), value)
            }
            DynCacheBackend::Redis { pool, ttl: _ } => {
                let Ok(mut conn) = pool.get().await else { return };
                let redis_key = format!(
                    "cache:{}:{}",
                    key.path,
                    key.token.as_deref().unwrap_or("")
                );
                if serialization::write_cached_response_hash(
                    &mut conn, &redis_key, &value,
                )
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
