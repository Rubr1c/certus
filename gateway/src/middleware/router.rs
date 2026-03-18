use std::sync::Arc;

use crate::server::state::app_state;

/// Builds a radix tree router of all the routes configured
/// and returns the router built
///
/// # Arguments
///
/// * `state` - arc of the AppState used to get the routes
#[inline(always)]
pub fn build_tree(
    state: Arc<app_state::AppState>,
) -> matchit::Router<Arc<str>> {
    let config = state.config.load();
    let route_conf = &config.routes;

    let mut router = matchit::Router::new();

    for route in route_conf.keys() {
        if let Err(e) = router.insert(route, Arc::<str>::from(route.as_str())) {
            tracing::error!("Failed to insert route '{}': {}", route, e);
        }

        let wildcard_route = if route == "/" {
            "/{*catchall}".to_string()
        } else {
            format!("{}/{{*catchall}}", route)
        };

        if let Err(e) =
            router.insert(wildcard_route, Arc::<str>::from(route.as_str()))
        {
            tracing::error!("Failed to insert route '{}': {}", route, e);
        }
    }

    router
}
