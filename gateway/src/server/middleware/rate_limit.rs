use std::time::Instant;

pub struct TokenBucket {
    pub tokens: f64,
    pub last_refill: Instant,
}

impl TokenBucket {
    pub fn new(max_tokens: f64) -> Self {
        Self { tokens: max_tokens, last_refill: Instant::now() }
    }
}
