use tower_http::cors::{self, CorsLayer};

#[inline(always)]
pub fn setup(mut app: axum::Router, origins: Vec<String>) -> axum::Router {
    app = if origins.is_empty() {
        tracing::warn!("No cors set allowing from all origins");
        app.layer(CorsLayer::new().allow_origin(cors::Any))
    } else {
        tracing::info!(
            origin_count = origins.len(),
            "Configured restricted CORS origins"
        );
        let parsed_origins: Vec<axum::http::HeaderValue> = origins
            .iter()
            .map(|ip| ip.parse().expect("Invalid Origin IP"))
            .collect();

        app.layer(CorsLayer::new().allow_origin(parsed_origins))
    };

    app
}
