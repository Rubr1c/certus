use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::Request as HttpRequest,
    routing::any,
};
use criterion::{Criterion, criterion_group, criterion_main};
use tokio::{net::TcpListener, runtime::Runtime};
use tower::ServiceExt;

use gateway::{
    config::cfg_utils::reload_config,
    server::{
        app_state::{self, AppState},
        middleware::router::{build_tree, reroute},
    },
};

fn create_runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}

async fn setup_state() -> Arc<AppState> {
    let config = reload_config("../examples/certus.config.yaml").await.unwrap();
    let state = Arc::new(AppState::new(config));
    build_tree(state.clone());
    app_state::init_server_state(state.clone()).await;
    state
}

async fn setup_server(state: Arc<AppState>) -> SocketAddr {
    let app = Router::new()
        .route("/{*any}", any(reroute))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    addr
}

fn bench_direct_call(c: &mut Criterion) {
    let rt = create_runtime();
    let state = rt.block_on(setup_state());

    c.bench_function("direct_call", |b| {
        b.to_async(&rt).iter(|| async {
            let req = Request::builder()
                .method("GET")
                .uri("/test")
                .body(Body::empty())
                .unwrap();

            let _ = reroute(State(state.clone()), req).await;
        });
    });
}

fn bench_tower_service(c: &mut Criterion) {
    let rt = create_runtime();
    let state = rt.block_on(setup_state());

    c.bench_function("tower_service", |b| {
        b.to_async(&rt).iter(|| async {
            let app = Router::new()
                .route("/{*any}", any(reroute))
                .with_state(state.clone());

            let req = HttpRequest::builder()
                .method("GET")
                .uri("/test")
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap();

            let _ = app.oneshot(req).await;
        });
    });
}

fn bench_http_integration(c: &mut Criterion) {
    let rt = create_runtime();
    let state = rt.block_on(setup_state());
    let addr = rt.block_on(setup_server(state));

    let client = reqwest::Client::new();
    let url = format!("http://{}/test", addr);

    c.bench_function("http_integration", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = client.get(&url).send().await.unwrap();
        });
    });
}

criterion_group!(benches, bench_direct_call, bench_tower_service, bench_http_integration);
criterion_main!(benches);
