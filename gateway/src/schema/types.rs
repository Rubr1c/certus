use std::{collections::HashMap, sync::Arc};

use hyper::{HeaderMap, Method, StatusCode, header};
use serde::Serialize;

pub struct ReqResSchemaDTO {
    pub full_path: Arc<str>,
    pub method: Method,
    pub query_params: Option<Arc<str>>,
    pub status_code: StatusCode,
    pub req_headers: HeaderMap,
    pub res_headers: HeaderMap,
    pub body_schema: Option<String>,
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
    pub body_schema: Option<String>,
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
            body_schema: self.body_schema,
        }
    }
}
