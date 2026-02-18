use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, atomic::AtomicUsize},
};

use crate::{
    config::RouteConfig,
    server::{
        middleware::load_balance::p2c_pick,
        upstream::{Protocol, UpstreamServer},
    },
};

#[test]
fn p2c_returns_correct_server() {
    let mut routes: HashMap<SocketAddr, Arc<UpstreamServer>> = HashMap::new();

    let addr1: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let addr2: SocketAddr = "127.0.0.2:3000".parse().unwrap();
    let default_addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();

    let mut upstream1 = UpstreamServer::new(addr1, 100, Protocol::HTTP1, false);
    let mut upstream2 = UpstreamServer::new(addr2, 100, Protocol::HTTP1, false);

    upstream1.active_connctions = AtomicUsize::new(6);
    upstream2.active_connctions = AtomicUsize::new(10);

    routes.insert(addr1, Arc::new(upstream1));
    routes.insert(addr2, Arc::new(upstream2));

    let config = RouteConfig {
        endpoints: vec![addr1, addr2],
        max_connections: 100,
        needs_auth: None,
        is_static: None,
        protocol: Protocol::HTTP1,
        token_weight: 1.0,
    };

    let target = p2c_pick(&routes, &config, &default_addr);

    assert_eq!(addr1, *target);
}
