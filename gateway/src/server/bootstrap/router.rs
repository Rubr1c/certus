use std::sync::Arc;

use axum::routing::any;

use crate::{middleware::pipeline, server::state::app_state::AppState};

#[inline]
pub fn run(
    interal_routes: axum::Router<Arc<AppState>>,
    app_state: Arc<AppState>,
) -> axum::Router {
    axum::Router::new()
        .nest("/_certus/api/v1", interal_routes)
        .route("/{*any}", any(pipeline::reroute))
        .with_state(app_state)
}
