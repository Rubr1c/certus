use std::{net::IpAddr, sync::Arc};

use chrono::{DateTime, Utc};
use serde::Serialize;

use super::serialize;

#[derive(Clone, Serialize)]
pub enum EarlyExit {
    RateLimited,
    Unauthorized,
    UpstreamError,
}

impl EarlyExit {
    #[inline(always)]
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
    pub method: hyper::Method,

    pub route: Arc<str>,
    pub upstream_addr: Option<Arc<str>>,
    pub early_exit: Option<EarlyExit>,
}

impl RequestMetric {
    #[inline(always)]
    fn new(
        route: Arc<str>,
        status_code: u16,
        duration_total_ms: u64,
        duration_upstream_ms: u64,
        bytes_in: u64,
        bytes_out: u64,
        client_ip: IpAddr,
        method: hyper::Method,
        upstream_addr: Option<Arc<str>>,
        early_exit: Option<EarlyExit>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            status_code,
            duration_total_ms,
            duration_upstream_ms,
            bytes_in,
            bytes_out,
            client_ip,
            method,
            route,
            upstream_addr,
            early_exit,
        }
    }

    #[inline(always)]
    pub fn rate_limited(
        route: Arc<str>,
        duration_total_ms: u64,
        bytes_in: u64,
        client_ip: IpAddr,
        method: hyper::Method,
    ) -> Self {
        Self::new(
            route,
            429,
            duration_total_ms,
            0,
            bytes_in,
            0,
            client_ip,
            method,
            None,
            Some(EarlyExit::RateLimited),
        )
    }

    #[inline(always)]
    pub fn unauthorized(
        route: Arc<str>,
        duration_total_ms: u64,
        bytes_in: u64,
        client_ip: IpAddr,
        method: hyper::Method,
    ) -> Self {
        Self::new(
            route,
            401,
            duration_total_ms,
            0,
            bytes_in,
            0,
            client_ip,
            method,
            None,
            Some(EarlyExit::Unauthorized),
        )
    }

    #[inline(always)]
    pub fn upstream_error(
        route: Arc<str>,
        status_code: u16,
        duration_total_ms: u64,
        duration_upstream_ms: u64,
        bytes_in: u64,
        client_ip: IpAddr,
        method: hyper::Method,
        upstream_addr: Arc<str>,
    ) -> Self {
        Self::new(
            route,
            status_code,
            duration_total_ms,
            duration_upstream_ms,
            bytes_in,
            0,
            client_ip,
            method,
            Some(upstream_addr),
            Some(EarlyExit::UpstreamError),
        )
    }

    #[inline(always)]
    pub fn success(
        route: Arc<str>,
        status_code: u16,
        duration_total_ms: u64,
        duration_upstream_ms: u64,
        bytes_in: u64,
        bytes_out: u64,
        client_ip: IpAddr,
        method: hyper::Method,
        upstream_addr: Arc<str>,
    ) -> Self {
        Self::new(
            route,
            status_code,
            duration_total_ms,
            duration_upstream_ms,
            bytes_in,
            bytes_out,
            client_ip,
            method,
            Some(upstream_addr),
            None,
        )
    }
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

impl CacheMetric {
    #[inline(always)]
    fn new(route: Arc<str>, result: CacheResult) -> Self {
        Self { route, timestamp: Utc::now(), result }
    }

    #[inline(always)]
    pub fn hit(route: Arc<str>) -> Self {
        Self::new(route, CacheResult::Hit)
    }

    #[inline(always)]
    pub fn miss(route: Arc<str>) -> Self {
        Self::new(route, CacheResult::Miss)
    }

    #[inline(always)]
    pub fn bypass(route: Arc<str>) -> Self {
        Self::new(route, CacheResult::Bypass)
    }
}

#[derive(Clone, Serialize)]
pub enum MetricEvent {
    Request(RequestMetric),
    Cache(CacheMetric),
}
