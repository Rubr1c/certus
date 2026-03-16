use std::{
    collections::HashMap,
    sync::{Arc, atomic::AtomicUsize},
};

use crate::{
    config::types::RouteConfig,
    middleware::load_balance::p2c::p2c_pick,
    upstream::{protocol::HttpVersion, server::UpstreamServer},
};

use super::create_addrs;

#[test]
fn p2c_returns_correct_server() {
    let mut routes: HashMap<String, Arc<UpstreamServer>> = HashMap::new();

    let addrs = create_addrs(3);

    let mut upstream1 =
        UpstreamServer::new(addrs[0].clone(), 100, HttpVersion::HTTP1);
    let mut upstream2 =
        UpstreamServer::new(addrs[1].clone(), 100, HttpVersion::HTTP1);

    upstream1.active_connctions = AtomicUsize::new(6);
    upstream2.active_connctions = AtomicUsize::new(10);

    routes.insert(addrs[0].clone(), Arc::new(upstream1));
    routes.insert(addrs[1].clone(), Arc::new(upstream2));

    let config = RouteConfig {
        endpoints: vec![addrs[0].clone(), addrs[1].clone()],
        token_weight: 1.0,
        ..RouteConfig::default()
    };

    let target = p2c_pick(&routes, &config, &addrs[2]);

    assert_eq!(addrs[0], *target);
}

#[test]
fn p2c_with_one_server() {
    let mut routes: HashMap<String, Arc<UpstreamServer>> = HashMap::new();

    let addrs = create_addrs(2);

    let upstream =
        UpstreamServer::new(addrs[0].clone(), 100, HttpVersion::HTTP1);

    routes.insert(addrs[0].clone(), Arc::new(upstream));

    let config = RouteConfig {
        endpoints: vec![addrs[0].clone()],
        token_weight: 1.0,
        ..RouteConfig::default()
    };

    let target = p2c_pick(&routes, &config, &addrs[1]);

    assert_eq!(addrs[0], *target);
}

#[test]
fn p2c_default_server() {
    let routes: HashMap<String, Arc<UpstreamServer>> = HashMap::new();

    let addrs = create_addrs(1);

    let config = RouteConfig { token_weight: 1.0, ..RouteConfig::default() };

    let target = p2c_pick(&routes, &config, &addrs[0]);

    assert_eq!(addrs[0], *target);
}
