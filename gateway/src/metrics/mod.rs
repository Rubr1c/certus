use std::{net::IpAddr, sync::Arc};

use chrono::{DateTime, Utc};
use hyper::Method;
use serde::{Serialize, Serializer};

fn serialize_method<S: Serializer>(
    method: &Method,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(method.as_str())
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

    #[serde(serialize_with = "serialize_method")]
    pub method: Method,

    pub route: Arc<str>,
    pub upstream_addr: Option<Arc<str>>,
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
