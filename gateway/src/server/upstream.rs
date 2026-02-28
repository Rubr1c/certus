use std::sync::{Arc, atomic::AtomicUsize};

use axum::body::Body;
use crossbeam::queue::SegQueue;
use hyper::client::conn;
use serde::{Deserialize, Serialize};

/// Enum representing if server is healthy or not
pub enum HealthState {
    Alive,
    Dead,
}

/// Enum for all available protocols
#[derive(Clone, Debug, Deserialize, Serialize, Default, Copy)]
pub enum HttpVersion {
    #[default]
    HTTP1,
    HTTP2,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub enum Protocol {
    #[default]
    HTTP,
    HTTPS,
}

/// Enum holding send request of the protocols
pub enum PooledConnection {
    Http1(conn::http1::SendRequest<Body>),
    Http2(conn::http2::SendRequest<Body>),
}

//TODO: add/remove nessesary/unesseseary fleids

/// Main server state holding all connection info
pub struct UpstreamServer {
    pub active_connctions: AtomicUsize,
    pub health_state: HealthState,
    pub pool: ConnectionPool,
}

/// Holds info for the connection pool of a server
pub struct ConnectionPool {
    pub server_addr: Arc<str>,
    pub hostname: String,
    pub http_version: HttpVersion,
    pub protocol: Protocol,
    pub max_connections: usize,
    pub total_connections: AtomicUsize,
    pub idle_connections: SegQueue<PooledConnection>,
}

/// Parses a server address string into (connect_addr, hostname, protocol).
///
/// Accepts formats like:
/// - `https://domain.com` → (`domain.com:443`, `domain.com`, HTTPS)
/// - `http://domain.com:8080` → (`domain.com:8080`, `domain.com`, HTTP)
/// - `domain.com:3000` → (`domain.com:3000`, `domain.com`, HTTP)
fn parse_address(address: &str) -> (String, String, Protocol) {
    let (protocol, rest) =
        if let Some(stripped) = address.strip_prefix("https://") {
            (Protocol::HTTPS, stripped)
        } else if let Some(stripped) = address.strip_prefix("http://") {
            (Protocol::HTTP, stripped)
        } else {
            (Protocol::HTTP, address)
        };

    let default_port = match protocol {
        Protocol::HTTPS => 443,
        Protocol::HTTP => 80,
    };

    let (hostname, connect_addr) =
        if let Some((host, port)) = rest.rsplit_once(':') {
            (host.to_string(), format!("{host}:{port}"))
        } else {
            (rest.to_string(), format!("{rest}:{default_port}"))
        };

    (connect_addr, hostname, protocol)
}

impl UpstreamServer {
    pub fn new(
        address: String,
        max_connections: usize,
        http_version: HttpVersion,
    ) -> Self {
        let (server_addr, hostname, protocol) = parse_address(&address);

        UpstreamServer {
            active_connctions: AtomicUsize::new(0),
            health_state: HealthState::Alive,
            pool: ConnectionPool {
                server_addr: Arc::<str>::from(server_addr),
                hostname,
                http_version,
                protocol,
                max_connections,
                total_connections: AtomicUsize::new(0),
                idle_connections: SegQueue::new(),
            },
        }
    }
}
