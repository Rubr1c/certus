use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

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
pub struct SerializableTokenBucket {
    pub tokens: f64,
    pub last_refill_epoch_ms: u64,
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
