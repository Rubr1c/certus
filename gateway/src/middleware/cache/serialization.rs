use bb8_redis::RedisConnectionManager;
use serde::{Deserialize, Serialize};

use super::response;

#[derive(Serialize, Deserialize)]
struct SerializableHeaders(Vec<(String, String)>);

#[inline(always)]
fn headers_to_json(headers: &hyper::HeaderMap) -> Option<String> {
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

#[inline(always)]
fn headers_from_json(json: &str) -> hyper::HeaderMap {
    let mut headers = hyper::HeaderMap::new();
    let Ok(data) = serde_json::from_str::<SerializableHeaders>(json) else {
        return headers;
    };
    for (k, v) in data.0 {
        if let (Ok(name), Ok(val)) = (
            k.parse::<axum::http::HeaderName>(),
            v.parse::<hyper::header::HeaderValue>(),
        ) {
            headers.insert(name, val);
        }
    }
    headers
}

#[inline(always)]
pub async fn write_cached_response_hash(
    conn: &mut bb8::PooledConnection<'_, RedisConnectionManager>,
    redis_key: &str,
    value: &response::CachedResponse,
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

#[inline(always)]
pub async fn read_cached_response_hash(
    conn: &mut bb8::PooledConnection<'_, RedisConnectionManager>,
    redis_key: &str,
) -> Option<response::CachedResponse> {
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

    Some(response::CachedResponse {
        status: hyper::StatusCode::from_u16(status).ok()?,
        headers: headers_from_json(&headers),
        body: axum::body::Bytes::from(body),
    })
}
