use axum::{body::Bytes, http::HeaderName};
use bb8_redis::RedisConnectionManager;
use hyper::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};

use crate::middleware::cache::response::CachedResponse;

#[derive(Serialize, Deserialize)]
struct SerializableHeaders(Vec<(String, String)>);

fn headers_to_json(headers: &HeaderMap) -> Option<String> {
    let data = SerializableHeaders(
        headers
            .iter()
            .map(|(k, v)| {
                (k.as_str().to_string(), v.to_str().unwrap_or("").to_string())
            })
            .collect(),
    );
    serde_json::to_string(&data).ok()
}

fn headers_from_json(json: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let Ok(data) = serde_json::from_str::<SerializableHeaders>(json) else {
        return headers;
    };
    for (k, v) in data.0 {
        if let (Ok(name), Ok(val)) =
            (k.parse::<HeaderName>(), v.parse::<hyper::header::HeaderValue>())
        {
            headers.insert(name, val);
        }
    }
    headers
}

pub async fn write_cached_response_hash(
    conn: &mut bb8::PooledConnection<'_, RedisConnectionManager>,
    redis_key: &str,
    value: &CachedResponse,
) -> Result<(), ()> {
    let Some(headers_json) = headers_to_json(&value.headers) else {
        return Err(());
    };
    redis::cmd("HSET")
        .arg(redis_key)
        .arg("status")
        .arg(value.status.as_u16())
        .arg("headers")
        .arg(headers_json)
        .arg("body")
        .arg(value.body.as_ref())
        .query_async(&mut **conn)
        .await
        .map_err(|_| ())
}

pub async fn read_cached_response_hash(
    conn: &mut bb8::PooledConnection<'_, RedisConnectionManager>,
    redis_key: &str,
) -> Option<CachedResponse> {
    let (status, headers, body): (
        Option<u16>,
        Option<String>,
        Option<Vec<u8>>,
    ) = redis::cmd("HMGET")
        .arg(redis_key)
        .arg("status")
        .arg("headers")
        .arg("body")
        .query_async(&mut **conn)
        .await
        .ok()?;

    let status = status?;
    let headers = headers?;
    let body = body?;

    Some(CachedResponse {
        status: StatusCode::from_u16(status).ok()?,
        headers: headers_from_json(&headers),
        body: Bytes::from(body),
    })
}
