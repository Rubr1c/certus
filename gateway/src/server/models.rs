use std::{net::SocketAddr, sync::atomic::AtomicUsize};

pub enum HealthState {
    Alive,
    Dead,
}

pub enum Protocol {
    HTTP1,
    HTTP2,
}

pub struct UpstreamServer {
    pub address: SocketAddr,
    pub active_connctions: AtomicUsize,
    pub max_connections: usize,
    pub health_state: HealthState,
    pub protocol: Protocol,
}

impl UpstreamServer {
    pub fn new(
        address: SocketAddr,
        max_connections: usize,
        protocol: Protocol,
    ) -> UpstreamServer {
        UpstreamServer {
            address,
            active_connctions: AtomicUsize::new(0),
            max_connections,
            health_state: HealthState::Alive,
            protocol,
        }
    }
}
