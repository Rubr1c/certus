use axum::http::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error(
        "Set GEMINI_API_KEY or GOOGLE_GENERATIVE_AI_API_KEY in the gateway environment to generate documentation."
    )]
    MissingProviderConfig,

    #[error("Invalid AI provider configuration: {0}")]
    ProviderConfig(String),

    #[error("{0}")]
    ProviderRequest(String),

    #[error("{0}")]
    ProviderResponse(String),

    #[error("Failed to serialize schemas")]
    Serialization(#[from] serde_json::Error),

    #[error("Schema query failed")]
    DataStore(String),

    #[error("Schema task panicked")]
    TaskJoin(String),
}

impl AiError {
    #[inline(always)]
    pub fn status_code(&self) -> StatusCode {
        match self {
            AiError::MissingProviderConfig | AiError::ProviderConfig(_) => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            AiError::ProviderRequest(_) | AiError::ProviderResponse(_) => {
                StatusCode::BAD_GATEWAY
            }
            AiError::Serialization(_)
            | AiError::DataStore(_)
            | AiError::TaskJoin(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    #[inline(always)]
    pub fn client_message(&self) -> String {
        match self {
            AiError::MissingProviderConfig
            | AiError::ProviderRequest(_)
            | AiError::ProviderResponse(_)
            | AiError::Serialization(_)
            | AiError::ProviderConfig(_) => self.to_string(),
            AiError::DataStore(_) => "Schema query failed".to_string(),
            AiError::TaskJoin(_) => "Schema task panicked".to_string(),
        }
    }
}
