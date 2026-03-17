use serde::Serialize;

#[derive(Serialize)]
pub struct RequestMetricRow {
    pub timestamp: String,
    pub route: String,
    pub status_code: u16,
    pub duration_total_ms: i64,
    pub duration_upstream_ms: i64,
    pub bytes_in: i64,
    pub bytes_out: i64,
    pub client_ip: String,
    pub method: String,
    pub upstream_addr: Option<String>,
    pub early_exit: Option<String>,
}

#[derive(Serialize)]
pub struct RequestMetricBucket {
    pub bucket: String,
    pub count: i64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: i64,
    pub max_duration_ms: i64,
    pub avg_upstream_ms: f64,
    pub bytes_in: i64,
    pub bytes_out: i64,
    pub status_2xx: i64,
    pub status_3xx: i64,
    pub status_4xx: i64,
    pub status_5xx: i64,
    pub error_count: i64,
}

#[derive(Serialize)]
pub struct RequestMetricSummary {
    pub key: String,
    pub count: i64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: i64,
    pub max_duration_ms: i64,
    pub avg_upstream_ms: f64,
    pub bytes_in: i64,
    pub bytes_out: i64,
    pub error_count: i64,
    pub status_2xx: i64,
    pub status_3xx: i64,
    pub status_4xx: i64,
    pub status_5xx: i64,
}
