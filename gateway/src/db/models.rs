use std::{collections::HashMap, sync::Arc};

use hyper::{HeaderMap, Method, StatusCode, header};
use serde::{Deserialize, Serialize};

/// Shape of log table in database
#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: i64,
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields: String,
}

impl LogEntry {
    /// Returns fields converted to hashmap
    pub fn get_fields(&self) -> HashMap<String, String> {
        serde_json::from_str(&self.fields).unwrap_or_default()
    }
}

pub struct ReqResSchemaDTO {
    pub full_path: Arc<str>,
    pub method: Method,
    pub query_params: Option<Arc<str>>,
    pub status_code: StatusCode,
    pub req_headers: HeaderMap,
    pub res_headers: HeaderMap,
}

#[derive(Serialize)]
pub struct ReqResSchema {
    pub full_path: String,
    pub method: String,
    pub query_params: Option<String>,
    pub status_code: u16,
    pub has_auth: bool,
    pub req_headers: String,
    pub res_headers: String,
}

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
pub struct CacheMetricRow {
    pub timestamp: String,
    pub route: String,
    pub result: String,
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
pub struct CacheMetricBucket {
    pub bucket: String,
    pub total: i64,
    pub hits: i64,
    pub misses: i64,
    pub bypasses: i64,
    pub hit_rate: f64,
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

impl ReqResSchemaDTO {
    pub fn into_s(self) -> ReqResSchema {
        const ALLOW: &[header::HeaderName] = &[
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::CACHE_CONTROL,
            header::LOCATION,
            header::ALLOW,
            header::RETRY_AFTER,
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            header::ACCESS_CONTROL_ALLOW_METHODS,
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            header::ACCESS_CONTROL_EXPOSE_HEADERS,
            header::ETAG,
        ];

        const SENSITIVE: &[header::HeaderName] =
            &[header::AUTHORIZATION, header::COOKIE, header::SET_COOKIE];

        let mut has_auth = false;

        let headers_to_json = |headers: &HeaderMap,
                               has_auth: &mut bool|
         -> String {
            let mut map = HashMap::with_capacity(headers.keys_len());

            for (k, v) in headers {
                if SENSITIVE.contains(k) || k.as_str().starts_with("x-auth") {
                    *has_auth = true;
                    continue;
                }
                if ALLOW.contains(k) || k.as_str().starts_with("x-") {
                    map.insert(
                        k.to_string(),
                        v.to_str().unwrap_or("").to_string(),
                    );
                }
            }

            serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())
        };

        let req_headers = headers_to_json(&self.req_headers, &mut has_auth);
        let res_headers = headers_to_json(&self.res_headers, &mut has_auth);

        ReqResSchema {
            full_path: self.full_path.to_string(),
            method: self.method.to_string(),
            query_params: self.query_params.map(|q| q.to_string()),
            status_code: self.status_code.as_u16(),
            has_auth,
            req_headers,
            res_headers,
        }
    }
}
