use std::{net::SocketAddr, sync::atomic::AtomicUsize};

use axum::body::Body;
use crossbeam::queue::SegQueue;
use hyper::client::conn;

pub enum HealthState {
    Alive,
    Dead,
}

#[derive(Clone)]
pub enum Protocol {
    HTTP1,
    HTTP2,
}

pub enum PooledConnection {
    Http1(conn::http1::SendRequest<Body>),
    Http2(conn::http2::SendRequest<Body>),
}

pub struct UpstreamServer {
    pub active_connctions: AtomicUsize,
    pub health_state: HealthState,
    pub pool: ConnectionPool,
}

pub struct ConnectionPool {
    pub server_addr: SocketAddr,
    pub protocol: Protocol,
    pub max_connections: usize,
    pub total_connections: AtomicUsize,
    pub idle_connections: SegQueue<PooledConnection>,
}

impl UpstreamServer {
    pub fn new(
        address: SocketAddr,
        max_connections: usize,
        protocol: Protocol,
    ) -> Self {
        UpstreamServer {
            active_connctions: AtomicUsize::new(0),
            health_state: HealthState::Alive,
            pool: ConnectionPool {
                server_addr: address,
                protocol,
                max_connections,
                total_connections: AtomicUsize::new(0),
                idle_connections: SegQueue::new(),
            },
        }
    }
}
