use serde::Serialize;

#[derive(Serialize)]
pub struct CacheMetricRow {
    pub timestamp: String,
    pub route: String,
    pub result: String,
}

#[derive(Serialize)]
pub struct CacheMetricBucket {
    pub bucket: String,
    pub total: i64,
    pub hits: i64,
    pub misses: i64,
    pub bypasses: i64,
    pub hit_rate: f64,
}
