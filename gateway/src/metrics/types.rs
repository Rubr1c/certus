use std::{net::IpAddr, sync::Arc};

use chrono::{DateTime, Utc};
use hyper::Method;
use serde::Serialize;

use super::serialize;

#[derive(Clone, Serialize)]
pub enum EarlyExit {
    RateLimited,
    Unauthorized,
    UpstreamError,
}

impl EarlyExit {
    pub fn as_str(&self) -> &'static str {
        match self {
            EarlyExit::RateLimited => "RateLimited",
            EarlyExit::Unauthorized => "Unauthorized",
            EarlyExit::UpstreamError => "UpstreamError",
        }
    }
}

#[derive(Clone, Serialize)]
pub struct RequestMetric {
    pub timestamp: DateTime<Utc>,
    pub status_code: u16,
    pub duration_total_ms: u64,
    pub duration_upstream_ms: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub client_ip: IpAddr,

    #[serde(serialize_with = "serialize::serialize_method")]
    pub method: Method,

    pub route: Arc<str>,
    pub upstream_addr: Option<Arc<str>>,
    pub early_exit: Option<EarlyExit>,
}

#[derive(Clone, Serialize)]
pub enum CacheResult {
    Hit,
    Miss,
    Bypass,
}

#[derive(Clone, Serialize)]
pub struct CacheMetric {
    pub route: Arc<str>,
    pub timestamp: DateTime<Utc>,
    pub result: CacheResult,
}

#[derive(Clone, Serialize)]
pub enum MetricEvent {
    Request(RequestMetric),
    Cache(CacheMetric),
}
