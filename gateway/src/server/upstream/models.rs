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

//TODO: add/remove nessesary/unesseseary fleids
pub struct UpstreamServer {
    pub active_connctions: AtomicUsize,
    pub health_state: HealthState,
    pub pool: ConnectionPool,
    pub req_auth: bool,
    // TODO: should not be in upstream but testing it for now here.
    pub token_weight: usize,
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
        req_auth: bool,
        token_weight: usize,
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
            req_auth,
            token_weight,
        }
    }
}
