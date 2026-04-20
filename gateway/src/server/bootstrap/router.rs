use std::sync::Arc;

use axum::routing::{any, get};

use crate::{
    middleware::pipeline,
    server::{bootstrap::dashboard_embed, state::app_state::AppState},
};

#[inline(always)]
pub fn run(
    interal_routes: axum::Router<Arc<AppState>>,
    app_state: Arc<AppState>,
) -> axum::Router {
    let dashboard =
        axum::Router::new().route("/{*rest}", get(dashboard_embed::serve));

    axum::Router::new()
        .nest("/_certus/api/v1", interal_routes)
        .route("/_certus/", get(dashboard_embed::index))
        .route("/_certus", get(dashboard_embed::index))
        .nest("/_certus", dashboard)
        .route("/{*any}", any(pipeline::reroute))
        .with_state(app_state)
}
