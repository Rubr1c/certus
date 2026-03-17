use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::response::IntoResponse;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::config::{
    AuthConfig, AuthType, CacheConfig, Config, ConnectionConfig,
    RateLimitConfig, RouteConfig, ServerConfig,
};
use crate::middleware::{pipeline, router};
use crate::server::state::{app_state, routing_table};
use crate::upstream::server;

use super::{
    test_args, test_db_conn, test_log_tx, test_metrics_tx, test_schema_tx,
};

async fn mock_upstream_ok(listener: TcpListener) {
    loop {
        let Ok((mut stream, _)) = listener.accept().await else {
            break;
        };
        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let _ = stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\nconnection: close\r\n\r\nok",
                )
                .await;
        });
    }
}

fn build_config(
    addr: &str,
    auth: AuthConfig,
    rate_limit: RateLimitConfig,
) -> Config {
    let mut routes = HashMap::new();
    routes.insert(
        "/api".to_string(),
        RouteConfig {
            endpoints: vec![addr.to_string()],
            needs_auth: auth.method != AuthType::None,
            token_weight: 1.0,
            ..RouteConfig::default()
        },
    );

    Config {
        server: ServerConfig::default(),
        auth,
        rate_limit,
        routes,
        default_server: addr.to_string(),
        connection: ConnectionConfig { connect_timeout: 5 },
        cache: CacheConfig { size: 100, ..CacheConfig::default() },
        tls: None,
    }
}

async fn build_state_with_upstream(
    config: Config,
    addr: &str,
) -> Arc<app_state::AppState> {
    let state = Arc::new(
        app_state::AppState::new(
            config,
            test_db_conn(),
            test_metrics_tx(),
            test_schema_tx(),
            test_log_tx(),
            None,
            test_args(),
        )
        .await,
    );

    let upstream = Arc::new(server::UpstreamServer::new(
        addr.to_string(),
        100,
        Default::default(),
    ));

    let mut routes_map = HashMap::new();
    routes_map.insert(addr.to_string(), upstream);

    let router = router::build_tree(state.clone());
    let table = routing_table::RoutingTable { router, routes: routes_map };
    state.routing_table.store(Arc::new(table));

    state
}

async fn call_reroute(
    state: Arc<app_state::AppState>,
    method: &str,
    path: &str,
    auth_header: Option<&str>,
) -> axum::response::Response {
    let client_addr: SocketAddr = "10.0.0.1:12345".parse().unwrap();

    let mut builder = hyper::Request::builder().method(method).uri(path);
    if let Some(h) = auth_header {
        builder = builder.header("Authorization", h);
    }
    let req = builder.body(axum::body::Body::empty()).unwrap();

    pipeline::reroute(
        axum::extract::State(state),
        axum::extract::ConnectInfo(client_addr),
        req,
    )
    .await
    .into_response()
}

#[tokio::test]
async fn reroute_forwards_to_upstream() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    tokio::spawn(mock_upstream_ok(listener));

    let config =
        build_config(&addr, AuthConfig::default(), RateLimitConfig::default());
    let state = build_state_with_upstream(config, &addr).await;

    let res = call_reroute(state, "GET", "/api", None).await;

    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn reroute_returns_not_found_for_unknown_path() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    tokio::spawn(mock_upstream_ok(listener));

    let config =
        build_config(&addr, AuthConfig::default(), RateLimitConfig::default());
    let state = build_state_with_upstream(config, &addr).await;

    let res = call_reroute(state, "GET", "/unknown", None).await;

    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn reroute_rate_limits() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    tokio::spawn(mock_upstream_ok(listener));

    let config = build_config(
        &addr,
        AuthConfig::default(),
        RateLimitConfig {
            max_tokens: 1.0,
            refill_rate: 0.0,
            ..RateLimitConfig::default()
        },
    );
    let state = build_state_with_upstream(config, &addr).await;

    let first = call_reroute(state.clone(), "GET", "/api", None).await;
    assert_eq!(first.status(), 200);

    let second = call_reroute(state, "GET", "/api", None).await;
    assert_eq!(second.status(), 429);
}

#[tokio::test]
async fn reroute_rejects_unauthorized() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    tokio::spawn(mock_upstream_ok(listener));

    let auth = AuthConfig {
        method: AuthType::JWT {
            secret: "test-secret".to_string(),
            algorithm: jsonwebtoken::Algorithm::default(),
        },
        prefix: "Bearer".to_string(),
    };

    let config = build_config(&addr, auth, RateLimitConfig::default());
    let state = build_state_with_upstream(config, &addr).await;

    let res = call_reroute(state, "GET", "/api", None).await;

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn reroute_caches_get_response() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    let accept_count = Arc::new(AtomicUsize::new(0));
    let counter = accept_count.clone();
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            counter.fetch_add(1, Ordering::SeqCst);
            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                let _ = stream.read(&mut buf).await;
                let _ = stream
                    .write_all(
                        b"HTTP/1.1 200 OK\r\ncontent-length: 6\r\nconnection: close\r\n\r\ncached",
                    )
                    .await;
            });
        }
    });

    let config =
        build_config(&addr, AuthConfig::default(), RateLimitConfig::default());
    let state = build_state_with_upstream(config, &addr).await;

    let first = call_reroute(state.clone(), "GET", "/api", None).await;
    assert_eq!(first.status(), 200);

    let second = call_reroute(state, "GET", "/api", None).await;
    assert_eq!(second.status(), 200);

    assert_eq!(accept_count.load(Ordering::SeqCst), 1);
}
