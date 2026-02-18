use std::net::{IpAddr, Ipv4Addr};
use std::thread;

use dashmap::DashMap;

use crate::config::{Config, RouteConfig};
use crate::server::middleware::rate_limit;

#[test]
fn allows_request_with_enough_tokens() {
    let config = Config::default();
    let route = RouteConfig::default();
    let tokens = DashMap::new();
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    let res = rate_limit::run(&route, ip, &config, &tokens);

    assert!(res.is_ok());
}

#[test]
fn rejects_when_tokens_exhausted() {
    let mut config = Config::default();
    config.rate_limit.max_tokens = 2.0;
    config.rate_limit.refill_rate = 0.0;

    let route = RouteConfig { token_weight: 1.0, ..RouteConfig::default() };
    let tokens = DashMap::new();
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    assert!(rate_limit::run(&route, ip, &config, &tokens).is_ok());
    assert!(rate_limit::run(&route, ip, &config, &tokens).is_ok());
    assert!(rate_limit::run(&route, ip, &config, &tokens).is_err());
}

#[test]
fn different_ips_get_separate_buckets() {
    let mut config = Config::default();
    config.rate_limit.max_tokens = 1.0;
    config.rate_limit.refill_rate = 0.0;

    let route = RouteConfig { token_weight: 1.0, ..RouteConfig::default() };
    let tokens = DashMap::new();

    let ip1 = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip2 = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));

    assert!(rate_limit::run(&route, ip1, &config, &tokens).is_ok());
    assert!(rate_limit::run(&route, ip1, &config, &tokens).is_err());

    assert!(rate_limit::run(&route, ip2, &config, &tokens).is_ok());
    assert!(rate_limit::run(&route, ip2, &config, &tokens).is_err());
}

#[test]
fn high_weight_drains_faster() {
    let mut config = Config::default();
    config.rate_limit.max_tokens = 10.0;
    config.rate_limit.refill_rate = 0.0;

    let route = RouteConfig { token_weight: 5.0, ..RouteConfig::default() };
    let tokens = DashMap::new();
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    assert!(rate_limit::run(&route, ip, &config, &tokens).is_ok());
    assert!(rate_limit::run(&route, ip, &config, &tokens).is_ok());
    assert!(rate_limit::run(&route, ip, &config, &tokens).is_err());
}

#[test]
fn tokens_refill_after_time() {
    let mut config = Config::default();
    config.rate_limit.max_tokens = 1.0;
    config.rate_limit.refill_rate = 10.0;

    let route = RouteConfig { token_weight: 1.0, ..RouteConfig::default() };
    let tokens = DashMap::new();
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    assert!(rate_limit::run(&route, ip, &config, &tokens).is_ok());
    assert!(rate_limit::run(&route, ip, &config, &tokens).is_err());

    thread::sleep(std::time::Duration::from_millis(150));

    assert!(rate_limit::run(&route, ip, &config, &tokens).is_ok());
}
