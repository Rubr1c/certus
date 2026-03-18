use bb8_redis::RedisConnectionManager;
use moka::sync::Cache;

use super::{
    key::{OwnedTokenBucketKey, TokenBucketKey, redis_key},
    token_bucket::{SerializableTokenBucket, TokenBucket},
};
use crate::{
    config::{RateLimitConfig, StorageType},
    connection,
};

pub enum DynBackend {
    InMemory(Cache<OwnedTokenBucketKey, TokenBucket>),
    Redis(bb8::Pool<RedisConnectionManager>),
}

#[inline(always)]
pub async fn build(config: &RateLimitConfig) -> DynBackend {
    match &config.rl_type {
        StorageType::InMemory => DynBackend::InMemory(
            Cache::builder()
                .time_to_idle(std::time::Duration::from_secs(3600))
                .max_capacity(100000)
                .build(),
        ),
        StorageType::Redis { url } => {
            DynBackend::Redis(connection::redis::create_pool(url).await)
        }
    }
}

impl DynBackend {
    pub async fn get(&self, key: &TokenBucketKey<'_>) -> Option<TokenBucket> {
        match self {
            DynBackend::InMemory(map) => map.get(&key.clone().into_owned()),
            DynBackend::Redis(pool) => {
                let mut conn = pool.get().await.ok()?;
                let redis_key = redis_key(key);
                let (tokens, last_refill_epoch_ms): (Option<f64>, Option<u64>) =
                    redis::cmd("HMGET")
                        .arg(&redis_key)
                        .arg("tokens")
                        .arg("last_refill_epoch_ms")
                        .query_async(&mut *conn)
                        .await
                        .ok()?;
                let serializable = SerializableTokenBucket {
                    tokens: tokens?,
                    last_refill_epoch_ms: last_refill_epoch_ms?,
                };
                Some(serializable.into())
            }
        }
    }

    pub async fn set(&self, key: TokenBucketKey<'_>, value: TokenBucket) {
        match self {
            DynBackend::InMemory(cache) => {
                cache.insert(key.into_owned(), value);
            }
            DynBackend::Redis(pool) => {
                let Ok(mut conn) = pool.get().await else { return };
                let serializable = SerializableTokenBucket::from(&value);
                let redis_key = redis_key(&key);
                let _: Result<(), _> = redis::cmd("HSET")
                    .arg(&redis_key)
                    .arg("tokens")
                    .arg(serializable.tokens)
                    .arg("last_refill_epoch_ms")
                    .arg(serializable.last_refill_epoch_ms)
                    .query_async(&mut *conn)
                    .await;
            }
        }
    }
}
