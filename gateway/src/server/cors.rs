use tower_http::cors::{self, CorsLayer};

#[inline]
pub fn setup(mut app: axum::Router, origins: Vec<String>) -> axum::Router {
    //TODO: make emit logs
    app = if origins.is_empty() {
        tracing::warn!("No cors set allowing from all origins");
        app.layer(CorsLayer::new().allow_origin(cors::Any))
    } else {
        let parsed_origins: Vec<axum::http::HeaderValue> = origins
            .iter()
            .map(|ip| ip.parse().expect("Invalid Origin IP"))
            .collect();

        app.layer(CorsLayer::new().allow_origin(parsed_origins))
    };

    app
}
