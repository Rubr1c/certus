use std::sync::atomic::AtomicUsize;

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
    pub server_addr: String,
    pub http_version: HttpVersion,
    pub max_connections: usize,
    pub total_connections: AtomicUsize,
    pub idle_connections: SegQueue<PooledConnection>,
}

impl UpstreamServer {
    pub fn new(
        address: String,
        max_connections: usize,
        http_version: HttpVersion,
    ) -> Self {
        UpstreamServer {
            active_connctions: AtomicUsize::new(0),
            health_state: HealthState::Alive,
            pool: ConnectionPool {
                server_addr: address,
                http_version,
                max_connections,
                total_connections: AtomicUsize::new(0),
                idle_connections: SegQueue::new(),
            },
        }
    }
}
