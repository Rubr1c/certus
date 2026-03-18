use std::time::Instant;

use crate::{
    config::{Config, RouteConfig},
    error::GatewayError,
    middleware::rate_limit,
};

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
    key: rate_limit::key::TokenBucketKey<'_>,
    config: &Config,
    backend: &rate_limit::backend::DynBackend,
) -> Result<(), GatewayError> {
    let max_tokens = config.rate_limit.max_tokens;
    let refill_rate = config.rate_limit.refill_rate;

    let mut bucket = backend.get(&key).await.unwrap_or_else(|| {
        rate_limit::token_bucket::TokenBucket::new(max_tokens)
    });

    let now = Instant::now();
    let duration = now.duration_since(bucket.last_refill).as_secs_f64();

    let tokens_to_add = duration * refill_rate;

    bucket.tokens = (bucket.tokens + tokens_to_add).min(max_tokens);
    bucket.last_refill = now;

    tracing::debug!(
        remaining = bucket.tokens,
        token_weight = target_route.token_weight,
        max_tokens,
        refill_rate,
        "Checking rate limit"
    );

    if bucket.tokens < target_route.token_weight {
        tracing::debug!(
            target = target_route.token_weight,
            remaining = bucket.tokens,
            max_tokens,
            refill_rate,
            "Rate limit exceeded"
        );
        backend.set(key, bucket).await;
        return Err(GatewayError::RateLimited);
    }
    bucket.tokens -= target_route.token_weight;
    tracing::debug!(
        remaining = bucket.tokens,
        removed = target_route.token_weight,
        "Removed {} tokens from bucket",
        target_route.token_weight
    );
    backend.set(key, bucket).await;
    Ok(())
}
