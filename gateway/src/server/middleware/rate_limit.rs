use std::{
    net::IpAddr,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use bb8_redis::RedisConnectionManager;
use dashmap::DashMap;
use redis::AsyncCommands;
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

#[derive(Hash, PartialEq, Eq)]
pub enum TokenBucketKey {
    Ip(IpAddr),
    Token(String),
}

fn redis_key(key: &TokenBucketKey) -> String {
    match key {
        TokenBucketKey::Ip(ip) => format!("rate_limit:ip:{}", ip),
        TokenBucketKey::Token(token) => format!("rate_limit:token:{}", token),
    }
}

pub enum DynRateLimitBackend {
    InMemory(DashMap<TokenBucketKey, TokenBucket>),
    Redis(bb8::Pool<RedisConnectionManager>),
}

impl DynRateLimitBackend {
    async fn get(&self, key: &TokenBucketKey) -> Option<TokenBucket> {
        match self {
            DynRateLimitBackend::InMemory(map) => {
                map.get(key).map(|v| v.clone())
            }
            DynRateLimitBackend::Redis(pool) => {
                let mut conn = pool.get().await.ok()?;
                let json: Option<String> =
                    conn.get(&redis_key(key)).await.ok()?;
                let s: SerializableTokenBucket =
                    serde_json::from_str(&json?).ok()?;
                Some(s.into())
            }
        }
    }

    async fn set(&self, key: TokenBucketKey, value: TokenBucket) {
        match self {
            DynRateLimitBackend::InMemory(map) => {
                map.insert(key, value);
            }
            DynRateLimitBackend::Redis(pool) => {
                let Ok(mut conn) = pool.get().await else { return };
                let Ok(json) = serde_json::to_string(
                    &SerializableTokenBucket::from(&value),
                ) else {
                    return;
                };
                let _: Result<(), _> = conn.set(&redis_key(&key), json).await;
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
    key: TokenBucketKey,
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
