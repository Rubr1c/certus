use std::{
    borrow::Cow,
    net::IpAddr,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use axum::http::HeaderValue;
use bb8_redis::RedisConnectionManager;
use moka::sync::Cache;
use serde::{Deserialize, Serialize};

use crate::{
    config::{Config, RouteConfig},
    server::error::GatewayError,
};

/// Holds the tokens remaining and last time bucket was refilled
#[derive(Clone)]
pub struct TokenBucket {
    pub tokens: f64,
    pub last_refill: Instant,
}

impl TokenBucket {
    pub fn new(max_tokens: f64) -> Self {
        Self { tokens: max_tokens, last_refill: Instant::now() }
    }
}

/// Serde-friendly intermediate for Redis storage.
/// Converts Instant to epoch millis since Instant doesn't impl Serialize.
#[derive(Serialize, Deserialize)]
struct SerializableTokenBucket {
    tokens: f64,
    last_refill_epoch_ms: u64,
}

impl From<&TokenBucket> for SerializableTokenBucket {
    fn from(b: &TokenBucket) -> Self {
        let elapsed_since_refill = b.last_refill.elapsed();
        let now_epoch =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        let refill_epoch = now_epoch.saturating_sub(elapsed_since_refill);
        Self {
            tokens: b.tokens,
            last_refill_epoch_ms: refill_epoch.as_millis() as u64,
        }
    }
}

impl From<SerializableTokenBucket> for TokenBucket {
    fn from(s: SerializableTokenBucket) -> Self {
        let now_epoch =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        let refill_epoch = Duration::from_millis(s.last_refill_epoch_ms);
        let elapsed = now_epoch.saturating_sub(refill_epoch);
        Self { tokens: s.tokens, last_refill: Instant::now() - elapsed }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum TokenBucketKey<'a> {
    Ip(IpAddr),
    Token(Cow<'a, str>),
    Header(Cow<'a, str>, HeaderValue),
}

type OwnedTokenBucketKey = TokenBucketKey<'static>;

impl TokenBucketKey<'_> {
    fn into_owned(self) -> OwnedTokenBucketKey {
        match self {
            TokenBucketKey::Ip(ip) => TokenBucketKey::Ip(ip),
            TokenBucketKey::Token(token) => {
                TokenBucketKey::Token(Cow::Owned(token.into_owned()))
            }
            TokenBucketKey::Header(header, value) => {
                TokenBucketKey::Header(Cow::Owned(header.into_owned()), value)
            }
        }
    }
}

fn redis_key(key: &TokenBucketKey<'_>) -> String {
    match key {
        TokenBucketKey::Ip(ip) => format!("rate_limit:ip:{}", ip),
        TokenBucketKey::Token(token) => format!("rate_limit:token:{}", token),
        TokenBucketKey::Header(header, value) => {
            format!("rate_limit:header:{}:{}", header, value.to_str().unwrap())
        }
    }
}

pub enum DynRateLimitBackend {
    InMemory(Cache<OwnedTokenBucketKey, TokenBucket>),
    Redis(bb8::Pool<RedisConnectionManager>),
}

impl DynRateLimitBackend {
    async fn get(&self, key: &TokenBucketKey<'_>) -> Option<TokenBucket> {
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

    async fn set(&self, key: TokenBucketKey<'_>, value: TokenBucket) {
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

/// Runs rate limiting by getting the bucket associated to the key
/// then fills the bucket based on time passed since last refill,
/// finally checks if user is rate limited for the target route and
/// removes tokens from bucket if not
///
/// # Errors
///
/// Returns an error if rate limit exceeded
#[inline]
pub async fn run(
    target_route: &RouteConfig,
    key: TokenBucketKey<'_>,
    config: &Config,
    backend: &DynRateLimitBackend,
) -> Result<(), GatewayError> {
    let max_tokens = config.rate_limit.max_tokens;
    let refill_rate = config.rate_limit.refill_rate;

    let mut bucket =
        backend.get(&key).await.unwrap_or_else(|| TokenBucket::new(max_tokens));

    let now = Instant::now();
    let duration = now.duration_since(bucket.last_refill).as_secs_f64();

    let tokens_to_add = duration * refill_rate;

    bucket.tokens = (bucket.tokens + tokens_to_add).min(max_tokens);
    bucket.last_refill = now;

    tracing::info!(remaining = bucket.tokens, "Checking Rate Limit");

    if bucket.tokens < target_route.token_weight {
        tracing::info!(
            target = target_route.token_weight,
            "Checking Rate Exceeded"
        );
        backend.set(key, bucket).await;
        return Err(GatewayError::RateLimited);
    }
    bucket.tokens -= target_route.token_weight;
    tracing::info!(
        remaining = bucket.tokens,
        "Removed {} tokens from bucket",
        target_route.token_weight
    );
    backend.set(key, bucket).await;
    Ok(())
}
