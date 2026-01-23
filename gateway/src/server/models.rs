use std::{hash::Hash, net::SocketAddr, sync::atomic::AtomicUsize};

use axum::{
    body::{Body, Bytes},
    response::{IntoResponse, Response},
};
use crossbeam::queue::SegQueue;
use hyper::{HeaderMap, Method, StatusCode, client::conn};

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

#[derive(Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub method: Method,
    pub user_id: Option<u64>,
    //TODO: get roles from config
    pub user_role: Option<String>,
    pub path: String,
}

#[derive(Clone)]
pub struct CachedResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}

impl IntoResponse for CachedResponse {
    fn into_response(self) -> Response {
        let mut response = Response::new(Body::from(self.body));

        *response.status_mut() = self.status;
        *response.headers_mut() = self.headers;

        response
    }
}
