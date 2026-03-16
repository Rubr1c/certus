use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{ConnectInfo, State};
use axum::response::IntoResponse;
use hyper::Request;
use parking_lot::Mutex;
use rusqlite::Connection;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};

use crate::config::{
    AuthConfig, AuthType, CacheConfig, CmdArgs, Config, ConnectionConfig,
    RateLimitConfig, RouteConfig, ServerConfig,
};
use crate::db::models::ReqResSchemaDTO;
use crate::logging::log_util::LogEntryDTO;
use crate::metrics::MetricEvent;
use crate::server::app_state::{AppState, RoutingTable};
use crate::server::upstream::UpstreamServer;

pub mod auth;
pub mod cache;
pub mod connection;
pub mod lb;
pub mod pipeline;
pub mod rate_limit;
pub mod reload;
pub mod router;

fn create_addrs(count: i32) -> Vec<String> {
    let mut addrs = Vec::<String>::new();

    for i in 0..count {
        addrs.push(format!("127.0.0.{}:3000", i));
    }

    addrs
}

fn test_db_conn() -> Arc<Mutex<Connection>> {
    Arc::new(Mutex::new(Connection::open_in_memory().unwrap()))
}

fn test_metrics_tx() -> mpsc::Sender<MetricEvent> {
    let (tx, _rx) = mpsc::channel::<MetricEvent>(16);
    tx
}

fn test_schema_tx() -> mpsc::Sender<ReqResSchemaDTO> {
    let (tx, _rx) = mpsc::channel::<ReqResSchemaDTO>(16);
    tx
}

fn test_log_tx() -> Option<broadcast::Sender<LogEntryDTO>> {
    None
}

fn test_args() -> Arc<CmdArgs> {
    Arc::new(CmdArgs { config: String::new(), save: false, ws: Vec::new() })
}

fn dummy_req() -> Request<Body> {
    Request::builder().body(Body::empty()).unwrap()
}

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
) -> Arc<AppState> {
    let state = Arc::new(
        AppState::new(
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

    let upstream = Arc::new(UpstreamServer::new(
        addr.to_string(),
        100,
        Default::default(),
    ));

    let mut routes_map = HashMap::new();
    routes_map.insert(addr.to_string(), upstream);

    let router = crate::server::middleware::router::build_tree(state.clone());
    let table = RoutingTable { router, routes: routes_map };
    state.routing_table.store(Arc::new(table));

    state
}

async fn call_reroute(
    state: Arc<AppState>,
    method: &str,
    path: &str,
    auth_header: Option<&str>,
) -> axum::response::Response {
    let client_addr: SocketAddr = "10.0.0.1:12345".parse().unwrap();

    let mut builder = Request::builder().method(method).uri(path);
    if let Some(h) = auth_header {
        builder = builder.header("Authorization", h);
    }
    let req = builder.body(Body::empty()).unwrap();

    crate::server::middleware::router::reroute(
        State(state),
        ConnectInfo(client_addr),
        req,
    )
    .await
    .into_response()
}
