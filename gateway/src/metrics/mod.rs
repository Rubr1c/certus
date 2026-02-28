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

    pub route: Arc<str>,
    pub upstream_addr: Option<Arc<str>>,
}

pub enum CacheResult {
    Hit,
    Miss,
    Bypass,
}

pub struct CacheMetric {
    pub route: Arc<str>,
    pub timestamp: DateTime<Utc>,
    pub result: CacheResult,
}

pub enum MetricEvent {
    Request(RequestMetric),
    Cache(CacheMetric),
}
