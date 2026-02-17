use std::{net::IpAddr, time::Instant};

use dashmap::DashMap;

use crate::{
    config::{Config, RouteConfig},
    server::error::GatewayError,
};

/// Holds the tokens remaining and last time bucket was refilled
pub struct TokenBucket {
    pub tokens: f64,
    pub last_refill: Instant,
}

impl TokenBucket {
    pub fn new(max_tokens: f64) -> Self {
        Self { tokens: max_tokens, last_refill: Instant::now() }
    }
}

/// Runs rate limiting by getting the bucket associated to the ip
/// then fills the bucket based on time passed based on last refill
/// finally checks if user is rate limited for the target route and
/// removes tokens from bucket if not
///
/// # Arguments
///
/// * `target_route` - config info of the target route
/// * `ip` - ip of the requester
/// * `config` - gateway config
/// * `state` - gateway app state
///
/// # Errors
///
/// Returns an error if:
/// * Rate limit exceeded
#[inline]
pub fn run(
    target_route: &RouteConfig,
    ip: IpAddr,
    config: &Config,
    user_tokens: &DashMap<IpAddr, TokenBucket>,
) -> Result<(), GatewayError> {
    let max_tokens = config.rate_limit.max_tokens;
    let refill_rate = config.rate_limit.refill_rate;

    let mut bucket_entry =
        user_tokens.entry(ip).or_insert(TokenBucket::new(max_tokens));

    let now = Instant::now();
    let duration = now.duration_since(bucket_entry.last_refill).as_secs_f64();

    let tokens_to_add = duration * refill_rate;

    bucket_entry.tokens = (bucket_entry.tokens + tokens_to_add).min(max_tokens);
    bucket_entry.last_refill = now;

    tracing::info!(remaining = bucket_entry.tokens, "Checking Rate Limit");

    if bucket_entry.tokens < target_route.token_weight {
        tracing::info!(
            target = target_route.token_weight,
            "Checking Rate Exceeded"
        );
        return Err(GatewayError::RateLimited);
    }
    bucket_entry.tokens -= target_route.token_weight;
    tracing::info!(
        remaining = bucket_entry.tokens,
        "Removed {} tokens from bucket",
        target_route.token_weight
    );
    drop(bucket_entry);
    Ok(())
}
