use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::Request as HttpRequest,
    routing::any,
};
use tokio::{net::TcpListener, time::Instant};
use tower::ServiceExt;

use crate::{
    config::cfg_utils::reload_config,
    server::{
        app_state::{self, AppState},
        middleware::router::{build_tree, reroute},
    },
};

/// Direct function call - just the handler
#[tokio::test]
async fn test_direct_call() {
    let config = reload_config("../examples/certus.config.yaml").await.unwrap();

    let state = Arc::new(AppState::new(config));

    build_tree(state.clone());
    app_state::init_server_state(state.clone()).await;

    let req = Request::builder()
        .method("GET")
        .uri("/test")
        .body(Body::empty())
        .unwrap();

    let start = Instant::now();

    let _ = reroute(State(state), req).await;

    let duration = start.elapsed();

    println!("[Direct] Time taken: {:?}", duration);
}

/// Tower service test - includes axum router/middleware stack, no network
#[tokio::test]
async fn test_tower_service() {
    let config = reload_config("../examples/certus.config.yaml").await.unwrap();

    let state = Arc::new(AppState::new(config));

    build_tree(state.clone());
    app_state::init_server_state(state.clone()).await;

    let app = Router::new()
        .route("/{*any}", any(reroute))
        .with_state(state);

    let req = HttpRequest::builder()
        .method("GET")
        .uri("/test")
        .header("Host", "localhost")
        .body(Body::empty())
        .unwrap();

    let start = Instant::now();

    let response = app.oneshot(req).await.unwrap();

    let duration = start.elapsed();

    // Read body to see error details
    let (parts, body) = response.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);

    println!("[Tower] Status: {}", parts.status);
    println!("[Tower] Time taken: {:?}", duration);
}

/// Full HTTP integration test - includes TCP + HTTP overhead
#[tokio::test]
async fn test_http_integration() {
    let config = reload_config("../examples/certus.config.yaml").await.unwrap();

    let state = Arc::new(AppState::new(config));

    build_tree(state.clone());
    app_state::init_server_state(state.clone()).await;

    let app = Router::new()
        .route("/{*any}", any(reroute))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();

    let start = Instant::now();

    let response = client
        .get(format!("http://{}/test", addr))
        .send()
        .await
        .unwrap();

    let duration = start.elapsed();

    println!("[HTTP] Status: {}", response.status());
    println!("[HTTP] Time taken: {:?}", duration);
}
