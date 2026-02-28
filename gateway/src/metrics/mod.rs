use std::{net::IpAddr, sync::Arc};

use chrono::{DateTime, Utc};
use hyper::Method;

pub struct RequestMetric {
    pub timestamp: DateTime<Utc>,
    pub status_code: u16,
    pub duration_total_ms: u64,
    pub duration_upstream_ms: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub client_ip: IpAddr,

    pub method: Method,

    pub route: Arc<String>,
    pub upstream_addr: Option<Arc<String>>,
}

pub struct CacheHitMetric {
    pub route: String,
    pub timestamp: DateTime<Utc>,
}

pub enum MetricEvent {
    Request(RequestMetric),
    CacheHit(CacheHitMetric),
}
