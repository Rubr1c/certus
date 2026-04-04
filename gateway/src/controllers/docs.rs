use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{db, schema::ReqResSchema, server::state::app_state};

const DEFAULT_MODEL: &str = "gemini-2.5-flash";
const SCHEMA_FETCH_CAP: u32 = 800;
const PROMPT_ROW_CAP: usize = 100;
const GEMINI_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/models";

#[derive(Serialize, Deserialize)]
pub struct GeneratedApiDocs {
    pub title: String,
    pub introduction: String,
    pub endpoints: Vec<EndpointDoc>,
}

#[derive(Serialize, Deserialize)]
pub struct EndpointDoc {
    pub path: String,
    pub method: String,
    pub status_code: u16,
    pub title: String,
    pub summary: String,
    pub markdown: String,
    pub limitations: String,
}

fn json_err(status: StatusCode, message: impl Into<String>) -> Response {
    (status, Json(serde_json::json!({ "error": message.into() })))
        .into_response()
}

fn dedupe_keep_newest(rows: Vec<ReqResSchema>) -> Vec<ReqResSchema> {
    let mut map: HashMap<(String, String, u16), ReqResSchema> = HashMap::new();
    for row in rows {
        let key = (row.full_path.clone(), row.method.clone(), row.status_code);
        map.entry(key).or_insert(row);
    }
    let mut v: Vec<_> = map.into_values().collect();
    v.sort_by(|a, b| {
        a.full_path
            .cmp(&b.full_path)
            .then_with(|| a.method.cmp(&b.method))
            .then_with(|| a.status_code.cmp(&b.status_code))
    });
    if v.len() > PROMPT_ROW_CAP {
        v.truncate(PROMPT_ROW_CAP);
    }
    v
}

fn gemini_api_key() -> Option<String> {
    std::env::var("GEMINI_API_KEY").ok().filter(|s| !s.is_empty()).or_else(
        || {
            std::env::var("GOOGLE_GENERATIVE_AI_API_KEY")
                .ok()
                .filter(|s| !s.is_empty())
        },
    )
}

fn gemini_model() -> String {
    std::env::var("GEMINI_MODEL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_MODEL.to_string())
}

fn build_prompt(records: &[ReqResSchema]) -> Result<String, serde_json::Error> {
    let json = serde_json::to_string_pretty(records)?;
    Ok(format!(
        r"You document HTTP traffic observed by an API gateway. You receive JSON records with fields: full_path, method, query_params, status_code, has_auth, req_headers, res_headers, body_schema (optional shallow JSON shape of response body when captured).

Rules:
- Describe ONLY what the records show. Do not invent routes, parameters, or response fields.
- When body_schema is null, say the response body was not captured (uncached traffic or non-JSON).
- Treat has_auth as an observation that auth-related headers were present, not as documentation of token format.
- req_headers/res_headers are filtered redacted JSON maps.

Records JSON:
{json}"
    ))
}

fn gemini_request_body(prompt: &str) -> serde_json::Value {
    serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": [{ "text": prompt }]
        }],
        "generationConfig": {
            "temperature": 0.2,
            "responseMimeType": "application/json",
            "responseSchema": {
                "type": "OBJECT",
                "properties": {
                    "title": { "type": "STRING", "description": "Short title for the API doc page" },
                    "introduction": { "type": "STRING", "description": "Overview paragraph" },
                    "endpoints": {
                        "type": "ARRAY",
                        "items": {
                            "type": "OBJECT",
                            "properties": {
                                "path": { "type": "STRING" },
                                "method": { "type": "STRING" },
                                "status_code": { "type": "INTEGER" },
                                "title": { "type": "STRING" },
                                "summary": { "type": "STRING" },
                                "markdown": { "type": "STRING", "description": "Markdown sections for this endpoint" },
                                "limitations": { "type": "STRING", "description": "What was not observed" }
                            },
                            "required": ["path", "method", "status_code", "title", "summary", "markdown", "limitations"]
                        }
                    }
                },
                "required": ["title", "introduction", "endpoints"]
            }
        }
    })
}

async fn call_gemini(
    api_key: &str,
    prompt: &str,
) -> Result<GeneratedApiDocs, String> {
    let model = gemini_model();
    let path = format!("{GEMINI_URL}/{model}:generateContent");
    let mut url = reqwest::Url::parse(&path).map_err(|e| e.to_string())?;
    url.query_pairs_mut().append_pair("key", api_key);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let res = client
        .post(url)
        .json(&gemini_request_body(prompt))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();
    let body: serde_json::Value =
        res.json().await.map_err(|e| e.to_string())?;

    if !status.is_success() {
        let msg = body
            .pointer("/error/message")
            .and_then(|m| m.as_str())
            .unwrap_or("Gemini request failed");
        return Err(format!("Gemini HTTP {status}: {msg}"));
    }

    let text = body
        .pointer("/candidates/0/content/parts/0/text")
        .and_then(|t| t.as_str())
        .ok_or_else(|| {
            let reason = body
                .pointer("/candidates/0/finishReason")
                .and_then(|r| r.as_str());
            format!("No text in Gemini response (finishReason={reason:?}).")
        })?;

    serde_json::from_str(text)
        .map_err(|e| format!("Failed to parse model JSON: {e}; text: {text}"))
}

#[inline(always)]
pub async fn generate(
    axum::extract::State(state): axum::extract::State<Arc<app_state::AppState>>,
) -> Response {
    let rows = match db::task::run(
        Arc::clone(&state.db_conn),
        move |conn| db::repository::schema::get_recent(conn, SCHEMA_FETCH_CAP),
        "Failed to read schemas for documentation",
        "Schema task panicked",
    )
    .await
    {
        Ok(r) => r,
        Err(_) => {
            return json_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Schema query failed",
            );
        }
    };

    let prepared = dedupe_keep_newest(rows);

    if prepared.is_empty() {
        return Json(GeneratedApiDocs {
            title: "API documentation".to_string(),
            introduction: "No request/response schemas are stored yet. Run traffic through the gateway with schema capture enabled; then try generating again.".to_string(),
            endpoints: vec![],
        })
        .into_response();
    }

    let Some(key) = gemini_api_key() else {
        return json_err(
            StatusCode::SERVICE_UNAVAILABLE,
            "Set GEMINI_API_KEY or GOOGLE_GENERATIVE_AI_API_KEY in the gateway environment to generate documentation.",
        );
    };

    let prompt = match build_prompt(&prepared) {
        Ok(p) => p,
        Err(_) => {
            return json_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to serialize schemas",
            );
        }
    };

    match call_gemini(&key, &prompt).await {
        Ok(doc) => Json(doc).into_response(),
        Err(e) => {
            tracing::warn!(err = %e, "Documentation generation failed");
            json_err(StatusCode::BAD_GATEWAY, e)
        }
    }
}
