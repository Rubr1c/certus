use axum::http::{Method, header};
use tower_http::cors::{self, CorsLayer};

#[inline(always)]
pub fn setup(mut app: axum::Router, origins: Vec<String>) -> axum::Router {
    let mut cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    cors = if origins.is_empty() {
        tracing::warn!("No cors set allowing from all origins");
        cors.allow_origin(cors::Any)
    } else {
        tracing::info!(
            origin_count = origins.len(),
            "Configured restricted CORS origins"
        );
        let parsed_origins: Vec<axum::http::HeaderValue> = origins
            .iter()
            .map(|ip| ip.parse().expect("Invalid Origin IP"))
            .collect();

        cors.allow_origin(parsed_origins)
    };

    app.layer(cors)
}
