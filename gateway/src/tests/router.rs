use std::sync::Arc;

use crate::config::{Config, RouteConfig};
use crate::server::app_state::AppState;
use crate::server::middleware::router;

use super::{test_db_conn, test_metrics_tx};

#[tokio::test]
async fn build_tree_matches_configured_route() {
    let mut config = Config::default();
    config.routes.insert("/api".to_string(), RouteConfig::default());

    let state = Arc::new(
        AppState::new(config, test_db_conn(), test_metrics_tx()).await,
    );
    let tree = router::build_tree(state);

    let matched = tree.at("/api").unwrap();

    assert_eq!(matched.value.as_str(), "/api");
}

#[tokio::test]
async fn build_tree_matches_wildcard_subpath() {
    let mut config = Config::default();
    config.routes.insert("/api".to_string(), RouteConfig::default());

    let state = Arc::new(
        AppState::new(config, test_db_conn(), test_metrics_tx()).await,
    );
    let tree = router::build_tree(state);

    let matched = tree.at("/api/users/123").unwrap();

    assert_eq!(matched.value.as_str(), "/api");
}

#[tokio::test]
async fn build_tree_no_match_returns_err() {
    let mut config = Config::default();
    config.routes.insert("/api".to_string(), RouteConfig::default());

    let state = Arc::new(
        AppState::new(config, test_db_conn(), test_metrics_tx()).await,
    );
    let tree = router::build_tree(state);

    let res = tree.at("/other");

    assert!(res.is_err());
}

#[tokio::test]
async fn build_tree_root_route() {
    let mut config = Config::default();
    config.routes.insert("/".to_string(), RouteConfig::default());

    let state = Arc::new(
        AppState::new(config, test_db_conn(), test_metrics_tx()).await,
    );
    let tree = router::build_tree(state);

    assert_eq!(tree.at("/").unwrap().value.as_str(), "/");
    assert_eq!(tree.at("/anything").unwrap().value.as_str(), "/");
}

#[tokio::test]
async fn build_tree_multiple_routes() {
    let mut config = Config::default();
    config.routes.insert("/api".to_string(), RouteConfig::default());
    config.routes.insert("/health".to_string(), RouteConfig::default());

    let state = Arc::new(
        AppState::new(config, test_db_conn(), test_metrics_tx()).await,
    );
    let tree = router::build_tree(state);

    assert_eq!(tree.at("/api").unwrap().value.as_str(), "/api");
    assert_eq!(tree.at("/health").unwrap().value.as_str(), "/health");
}
