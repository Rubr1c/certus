use axum::response::IntoResponse;
use thiserror::Error;

/// Error enum for any error that may happen related in the gateway
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

    #[error("Internal Server Error")]
    InternalServerError,
}

//TODO: make errors trace here
impl IntoResponse for GatewayError {
    /// Turns the GatewayError into a response that can be
    /// returned from the server with a status and message
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match &self {
            GatewayError::Overloaded => {
                (axum::http::StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
            GatewayError::ConnectionFailed(_) => {
                (axum::http::StatusCode::BAD_GATEWAY, self.to_string())
            }
            GatewayError::NotFound => {
                (axum::http::StatusCode::NOT_FOUND, self.to_string())
            }
            GatewayError::Io(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ),
            GatewayError::Unauthorized => {
                (axum::http::StatusCode::UNAUTHORIZED, self.to_string())
            }
            GatewayError::RateLimited => {
                (axum::http::StatusCode::TOO_MANY_REQUESTS, self.to_string())
            }
            GatewayError::InternalServerError => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                self.to_string(),
            ),
        };

        (status, error_message).into_response()
    }
}
