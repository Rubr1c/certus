use std::net::IpAddr;

use serde::Deserialize;

pub const INVALID_INTERVAL_MSG: &str =
    "Invalid interval. Use: 1m, 5m, 15m, 30m, 1h, 6h, 1d";
pub const INVALID_GROUP_BY_MSG: &str =
    "Invalid group_by. Use: route, method, status, upstream";

#[derive(Deserialize)]
pub struct BaseMetricQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub route: Option<String>,
}

#[derive(Deserialize)]
pub struct RequestMetricQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub route: Option<String>,
    pub status: Option<u16>,
    pub ip: Option<IpAddr>,
    pub method: Option<String>,
}

#[derive(Deserialize)]
pub struct AggregateRequestQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub route: Option<String>,
    pub status: Option<u16>,
    pub method: Option<String>,
    pub interval: Option<String>,
}

#[derive(Deserialize)]
pub struct AggregateCacheQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub route: Option<String>,
    pub interval: Option<String>,
}

#[derive(Deserialize)]
pub struct SummaryQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub group_by: Option<String>,
}

#[inline(always)]
pub fn interval(s: &str) -> Option<i64> {
    match s {
        "1m" => Some(60),
        "5m" => Some(300),
        "15m" => Some(900),
        "30m" => Some(1800),
        "1h" => Some(3600),
        "6h" => Some(21600),
        "1d" => Some(86400),
        _ => None,
    }
}

#[inline(always)]
pub fn group_by(group_by: &str) -> bool {
    matches!(group_by, "route" | "method" | "status" | "upstream")
}
