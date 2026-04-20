use std::time::Duration;

use reqwest::Url;

use crate::{ai::docs::types::GeneratedApiDocs, ai::error::AiError};

const DEFAULT_MODEL: &str = "gemini-2.5-flash";
const GEMINI_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/models";

pub struct GeminiProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl GeminiProvider {
    #[inline(always)]
    pub fn from_env() -> Result<Self, AiError> {
        let api_key = std::env::var("GEMINI_API_KEY")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                std::env::var("GOOGLE_GENERATIVE_AI_API_KEY")
                    .ok()
                    .filter(|s| !s.is_empty())
            })
            .ok_or(AiError::MissingProviderConfig)?;

        let model = std::env::var("GEMINI_MODEL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| AiError::ProviderConfig(e.to_string()))?;

        Ok(Self { api_key, model, client })
    }

    fn request_body(prompt: &str) -> serde_json::Value {
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
                        "highlights": {
                            "type": "ARRAY",
                            "items": { "type": "STRING" },
                            "description": "Top-level bullets summarizing notable observed behavior"
                        },
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
                                    "observed_request": {
                                        "type": "ARRAY",
                                        "items": { "type": "STRING" }
                                    },
                                    "observed_response": {
                                        "type": "ARRAY",
                                        "items": { "type": "STRING" }
                                    },
                                    "markdown": {
                                        "type": "STRING",
                                        "description": "Optional markdown sections for backwards compatibility"
                                    },
                                    "limitations": {
                                        "type": "ARRAY",
                                        "items": { "type": "STRING" },
                                        "description": "What was not observed"
                                    }
                                },
                                "required": [
                                    "path",
                                    "method",
                                    "status_code",
                                    "title",
                                    "summary",
                                    "observed_request",
                                    "observed_response",
                                    "limitations"
                                ]
                            }
                        }
                    },
                    "required": ["title", "introduction", "endpoints"]
                }
            }
        })
    }

    #[inline(always)]
    pub async fn generate(
        &self,
        prompt: &str,
    ) -> Result<GeneratedApiDocs, AiError> {
        let path = format!("{GEMINI_URL}/{}:generateContent", self.model);
        let mut url = Url::parse(&path)
            .map_err(|e| AiError::ProviderConfig(e.to_string()))?;
        url.query_pairs_mut().append_pair("key", &self.api_key);

        let res = self
            .client
            .post(url)
            .json(&Self::request_body(prompt))
            .send()
            .await
            .map_err(|e| AiError::ProviderRequest(e.to_string()))?;

        let status = res.status();
        let body: serde_json::Value = res
            .json()
            .await
            .map_err(|e| AiError::ProviderResponse(e.to_string()))?;

        if !status.is_success() {
            let msg = body
                .pointer("/error/message")
                .and_then(|m| m.as_str())
                .unwrap_or("Gemini request failed");
            return Err(AiError::ProviderRequest(format!(
                "Gemini HTTP {status}: {msg}"
            )));
        }

        let text = body
            .pointer("/candidates/0/content/parts/0/text")
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                let reason = body
                    .pointer("/candidates/0/finishReason")
                    .and_then(|r| r.as_str());
                AiError::ProviderResponse(format!(
                    "No text in Gemini response (finishReason={reason:?})."
                ))
            })?;

        serde_json::from_str(text).map_err(|e| {
            AiError::ProviderResponse(format!(
                "Failed to parse model JSON: {e}; text: {text}"
            ))
        })
    }
}
