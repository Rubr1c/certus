use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request as HttpRequest, StatusCode, Version},
    routing::any,
};
use criterion::{Criterion, criterion_group, criterion_main};
use hyper_util::{
    client::legacy::{Client, connect::HttpConnector},
    rt::TokioExecutor,
};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use parking_lot::Mutex;
use rusqlite::Connection;
use tokio::{
    net::TcpListener,
    runtime::Runtime,
    sync::mpsc,
    time::{Instant, sleep},
};

use gateway::{
    cli::CmdArgs,
    config::{
        AuthConfig, AuthType, CacheConfig, Config, ConnectionConfig,
        RateLimitConfig, RateLimitKey, RouteConfig, ServerConfig,
        StaticCacheConfig, StorageType,
    },
    metrics::MetricEvent,
    middleware::pipeline,
    schema::ReqResSchemaDTO,
    server::state::{app_state::AppState, initializer},
    upstream::protocol::HttpVersion,
};

const AUTH_SECRET: &str = "bench-jwt-secret";
const MAX_BODY_BYTES: usize = 1024 * 1024;

type HttpClient = Client<HttpConnector, Body>;

#[derive(serde::Serialize)]
struct BenchClaims {
    user_id: String,
    role: String,
    exp: usize,
}

struct BenchFixture {
    client: HttpClient,
    base_uri: String,
    auth_header: String,
}

fn create_runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime")
}

fn make_route(
    path_endpoints: &[&str],
    no_cache: bool,
    needs_auth: bool,
) -> RouteConfig {
    RouteConfig {
        endpoints: path_endpoints.iter().map(|v| (*v).to_string()).collect(),
        no_cache,
        needs_auth,
        http_version: HttpVersion::HTTP1,
        token_weight: 1.0,
        ..RouteConfig::default()
    }
}

fn build_config() -> Config {
    let mut routes = HashMap::new();
    let upstreams = ["127.0.0.1:3101", "127.0.0.1:3102"];

    routes.insert(
        "/api/public/text".to_string(),
        make_route(&upstreams, true, false),
    );
    routes
        .insert("/api/users".to_string(), make_route(&upstreams, false, false));
    routes.insert(
        "/api/private/profile".to_string(),
        make_route(&upstreams, false, true),
    );

    Config {
        server: ServerConfig::default(),
        routes,
        tls: None,
        auth: AuthConfig {
            method: AuthType::JWT {
                secret: AUTH_SECRET.to_string(),
                algorithm: Algorithm::HS256,
            },
            prefix: "Bearer".to_string(),
        },
        rate_limit: RateLimitConfig {
            max_tokens: 1_000_000.0,
            refill_rate: 1_000_000.0,
            key: RateLimitKey::Ip,
            rl_type: StorageType::InMemory,
        },
        connection: ConnectionConfig { connect_timeout: 2 },
        cache: CacheConfig {
            size: 2048,
            cache_type: StorageType::InMemory,
            ttl: Some(60),
            tti: Some(30),
            max_size: 65_536,
            static_cache: StaticCacheConfig {
                cache_type: StorageType::InMemory,
            },
        },
        default_server: "127.0.0.1:3101".to_string(),
    }
}

fn build_auth_header() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs();

    let claims = BenchClaims {
        user_id: "bench-user-1".to_string(),
        role: "bench-admin".to_string(),
        exp: (now + 3600) as usize,
    };

    let token = jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(AUTH_SECRET.as_bytes()),
    )
    .expect("failed to sign auth token");

    format!("Bearer {token}")
}

fn build_request(
    method: Method,
    uri: &str,
    auth_header: Option<&str>,
) -> HttpRequest<Body> {
    let mut builder = HttpRequest::builder()
        .method(method)
        .uri(uri)
        .version(Version::HTTP_11)
        .header("Host", "localhost");

    if let Some(auth) = auth_header {
        builder = builder.header("Authorization", auth);
    }

    builder.body(Body::empty()).expect("failed to build benchmark request")
}

async fn wait_for_ready_ok(client: &HttpClient, uri: &str, timeout: Duration) {
    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        let req = build_request(Method::GET, uri, None);

        if let Ok(res) = client.request(req).await {
            let status = res.status();
            let _ = to_bytes(Body::new(res.into_body()), MAX_BODY_BYTES).await;

            if status.is_success() {
                return;
            }
        }

        sleep(Duration::from_millis(200)).await;
    }

    panic!("timed out waiting for endpoint readiness: {uri}");
}

async fn send_request_expect_ok(
    client: &HttpClient,
    method: Method,
    uri: &str,
    auth_header: Option<&str>,
) {
    let req = build_request(method, uri, auth_header);
    let res = client.request(req).await.expect("request failed");
    let status = res.status();
    let _ = to_bytes(Body::new(res.into_body()), MAX_BODY_BYTES).await;

    assert_eq!(status, StatusCode::OK, "unexpected status for {uri}");
}

async fn setup_state() -> Arc<AppState> {
    let conn = Arc::new(Mutex::new(
        Connection::open_in_memory().expect("failed to open sqlite in memory"),
    ));

    let (metrics_tx, mut metrics_rx) = mpsc::channel::<MetricEvent>(4096);
    let (schema_tx, mut schema_rx) = mpsc::channel::<ReqResSchemaDTO>(4096);

    tokio::spawn(async move { while metrics_rx.recv().await.is_some() {} });
    tokio::spawn(async move { while schema_rx.recv().await.is_some() {} });

    let args = Arc::new(CmdArgs {
        config: String::new(),
        save: false,
        ws: Vec::new(),
    });

    let state = Arc::new(
        AppState::new(
            build_config(),
            conn,
            metrics_tx,
            schema_tx,
            None,
            None,
            args,
        )
        .await,
    );

    initializer::init(state.clone()).await;
    state
}

async fn setup_gateway(state: Arc<AppState>) -> SocketAddr {
    let app = Router::new()
        .route("/{*any}", any(pipeline::reroute))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind gateway benchmark listener");
    let addr = listener
        .local_addr()
        .expect("failed to read gateway benchmark listener addr");

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("gateway benchmark server exited unexpectedly");
    });

    addr
}

async fn setup_fixture() -> BenchFixture {
    let readiness_client: HttpClient =
        Client::builder(TokioExecutor::new()).build_http();

    wait_for_ready_ok(
        &readiness_client,
        "http://127.0.0.1:3101/api/public/text",
        Duration::from_secs(30),
    )
    .await;
    wait_for_ready_ok(
        &readiness_client,
        "http://127.0.0.1:3102/api/users",
        Duration::from_secs(30),
    )
    .await;

    let state = setup_state().await;
    let gateway_addr = setup_gateway(state).await;

    let client: HttpClient = Client::builder(TokioExecutor::new()).build_http();
    let base_uri = format!("http://{gateway_addr}");

    wait_for_ready_ok(
        &client,
        &format!("{base_uri}/api/public/text"),
        Duration::from_secs(10),
    )
    .await;

    BenchFixture { client, base_uri, auth_header: build_auth_header() }
}

fn bench_http1_pipeline_public_uncached_get(c: &mut Criterion) {
    let rt = create_runtime();
    let fixture = rt.block_on(setup_fixture());
    let client = fixture.client.clone();
    let uri = format!("{}/api/public/text", fixture.base_uri);

    c.bench_function("http1_pipeline_public_uncached_get", |b| {
        b.to_async(&rt).iter(|| async {
            send_request_expect_ok(&client, Method::GET, &uri, None).await;
        });
    });
}

fn bench_http1_pipeline_users_cached_hit(c: &mut Criterion) {
    let rt = create_runtime();
    let fixture = rt.block_on(setup_fixture());
    let client = fixture.client.clone();
    let uri = format!("{}/api/users", fixture.base_uri);

    rt.block_on(async {
        send_request_expect_ok(&client, Method::GET, &uri, None).await;
    });

    c.bench_function("http1_pipeline_users_cached_hit", |b| {
        b.to_async(&rt).iter(|| async {
            send_request_expect_ok(&client, Method::GET, &uri, None).await;
        });
    });
}

fn bench_http1_pipeline_private_auth_get(c: &mut Criterion) {
    let rt = create_runtime();
    let fixture = rt.block_on(setup_fixture());
    let client = fixture.client.clone();
    let uri = format!("{}/api/private/profile", fixture.base_uri);
    let auth_header = fixture.auth_header;

    c.bench_function("http1_pipeline_private_auth_get", |b| {
        b.to_async(&rt).iter(|| async {
            send_request_expect_ok(
                &client,
                Method::GET,
                &uri,
                Some(auth_header.as_str()),
            )
            .await;
        });
    });
}

criterion_group!(
    benches,
    bench_http1_pipeline_public_uncached_get,
    bench_http1_pipeline_users_cached_hit,
    bench_http1_pipeline_private_auth_get
);
criterion_main!(benches);
