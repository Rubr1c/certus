use hyper::header::CONTENT_TYPE;

use super::extractor;

pub fn extract_body(headers: &hyper::HeaderMap, body: &[u8]) -> Option<String> {
    let is_json = headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("application/json"))
        .unwrap_or(false);

    if !is_json {
        return None;
    }

    extractor::extract_body_schema(body)
}
