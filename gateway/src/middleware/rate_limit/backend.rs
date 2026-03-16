use bb8_redis::RedisConnectionManager;
use moka::sync::Cache;

use super::{
    key::{OwnedTokenBucketKey, TokenBucketKey, redis_key},
    token_bucket::{SerializableTokenBucket, TokenBucket},
};

pub enum DynRateLimitBackend {
    InMemory(Cache<OwnedTokenBucketKey, TokenBucket>),
    Redis(bb8::Pool<RedisConnectionManager>),
}

impl DynRateLimitBackend {
    pub async fn get(&self, key: &TokenBucketKey<'_>) -> Option<TokenBucket> {
        match self {
            DynRateLimitBackend::InMemory(map) => {
                map.get(&key.clone().into_owned())
            }
            DynRateLimitBackend::Redis(pool) => {
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
            DynRateLimitBackend::InMemory(cache) => {
                cache.insert(key.into_owned(), value);
            }
            DynRateLimitBackend::Redis(pool) => {
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
