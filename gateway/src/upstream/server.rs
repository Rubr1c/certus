use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicUsize},
};

use crossbeam::queue::SegQueue;

use crate::upstream::protocol;

use super::address;

/// Enum representing if server is healthy or not
#[repr(u8)]
pub enum HealthState {
    Alive = 0,
    Dead = 1,
}

/// Main server state holding all connection info
pub struct UpstreamServer {
    pub active_connctions: AtomicUsize,
    pub health_state: AtomicU8,
    pub pool: ConnectionPool,
}

/// Holds info for the connection pool of a server
pub struct ConnectionPool {
    pub server_addr: Arc<str>,
    pub hostname: String,
    pub http_version: protocol::HttpVersion,
    pub protocol: protocol::Protocol,
    pub max_connections: usize,
    pub total_connections: AtomicUsize,
    pub idle_connections: SegQueue<protocol::PooledConnection>,
}

impl UpstreamServer {
    pub fn new(
        address: String,
        max_connections: usize,
        http_version: protocol::HttpVersion,
    ) -> Self {
        let (server_addr, hostname, protocol) =
            address::parse_address(&address);

        UpstreamServer {
            active_connctions: AtomicUsize::new(0),
            health_state: AtomicU8::new(HealthState::Alive as u8),
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
