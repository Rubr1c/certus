use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("Upstream overloaded")]
    Overloaded,

    #[error("Failed to connect to upstream: {0}")]
    ConnectionFailed(String),

    #[error("No route found")]
    NotFound,

    #[error("Internal IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Rate Limited")]
    RateLimited,
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            GatewayError::Overloaded => {
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
            GatewayError::ConnectionFailed(_) => {
                (StatusCode::BAD_GATEWAY, self.to_string())
            }
            GatewayError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            GatewayError::Io(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ),
            GatewayError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }
            GatewayError::RateLimited => {
                (StatusCode::TOO_MANY_REQUESTS, self.to_string())
            }
        };

        (status, error_message).into_response()
    }
}
