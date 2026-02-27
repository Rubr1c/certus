use chrono::{DateTime, Utc};

pub struct RequestMetric {
    pub route: String,
    pub status_code: u16,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

pub struct CacheHitMetric {
    pub route: String,
    pub timestamp: DateTime<Utc>,
}

pub enum MetricEvent {
    Request(RequestMetric),
    CacheHit(CacheHitMetric),
}
