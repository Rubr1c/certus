use std::{collections::HashMap, sync::Arc};

use hyper::HeaderMap;
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
    pub route: Arc<str>,
    pub req_headers: HeaderMap,
    pub res_headers: HeaderMap,
}

pub struct ReqResSchema {
    pub route: String,
    pub req_headers: String,
    pub res_headers: String,
}

impl ReqResSchemaDTO {
    //TODO: filter out some things and make it configable too
    //      for senstive or useless fields. also add query params?
    //      not sure what are in the headers and what are not.
    pub fn into_s(self) -> ReqResSchema {
        let headers_to_json = |headers: &HeaderMap| -> String {
            let mut map = HashMap::with_capacity(headers.keys_len());

            for (k, v) in headers {
                map.insert(k.to_string(), v.to_str().unwrap_or("").to_string());
            }

            serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())
        };

        ReqResSchema {
            route: self.route.to_string(),
            req_headers: headers_to_json(&self.req_headers),
            res_headers: headers_to_json(&self.res_headers),
        }
    }
}
