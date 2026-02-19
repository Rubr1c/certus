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

use super::create_socket_addr;

#[test]
fn p2c_returns_correct_server() {
    let mut routes: HashMap<SocketAddr, Arc<UpstreamServer>> = HashMap::new();

    let addrs = create_socket_addr(3);

    let mut upstream1 = UpstreamServer::new(addrs[0], 100, Protocol::HTTP1);
    let mut upstream2 = UpstreamServer::new(addrs[1], 100, Protocol::HTTP1);

    upstream1.active_connctions = AtomicUsize::new(6);
    upstream2.active_connctions = AtomicUsize::new(10);

    routes.insert(addrs[0], Arc::new(upstream1));
    routes.insert(addrs[1], Arc::new(upstream2));

    let config = RouteConfig {
        endpoints: vec![addrs[0], addrs[1]],
        max_connections: 100,
        needs_auth: false,
        is_static: false,
        protocol: Protocol::HTTP1,
        token_weight: 1.0,
    };

    let target = p2c_pick(&routes, &config, &addrs[2]);

    assert_eq!(addrs[0], *target);
}

#[test]
fn p2c_with_one_server() {
    let mut routes: HashMap<SocketAddr, Arc<UpstreamServer>> = HashMap::new();

    let addrs = create_socket_addr(2);

    let upstream = UpstreamServer::new(addrs[0], 100, Protocol::HTTP1);

    routes.insert(addrs[0], Arc::new(upstream));

    let config = RouteConfig {
        endpoints: vec![addrs[0]],
        max_connections: 100,
        needs_auth: false,
        is_static: false,
        protocol: Protocol::HTTP1,
        token_weight: 1.0,
    };

    let target = p2c_pick(&routes, &config, &addrs[1]);

    assert_eq!(addrs[0], *target);
}

#[test]
fn p2c_default_server() {
    let routes: HashMap<SocketAddr, Arc<UpstreamServer>> = HashMap::new();

    let addrs = create_socket_addr(1);

    let config = RouteConfig {
        endpoints: vec![],
        max_connections: 100,
        needs_auth: false,
        is_static: false,
        protocol: Protocol::HTTP1,
        token_weight: 1.0,
    };

    let target = p2c_pick(&routes, &config, &addrs[0]);

    assert_eq!(addrs[0], *target);
}
