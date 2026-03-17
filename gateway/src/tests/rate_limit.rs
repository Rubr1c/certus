use std::net::{IpAddr, Ipv4Addr};

use moka::sync::Cache;

use crate::{
    config::{Config, RouteConfig},
    middleware::rate_limit,
};

#[tokio::test]
async fn allows_request_with_enough_tokens() {
    let config = Config::default();
    let route = RouteConfig::default();
    let backend = rate_limit::backend::DynBackend::InMemory(Cache::new(100));
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    let res = rate_limit::limiter::run(
        &route,
        rate_limit::key::TokenBucketKey::Ip(ip),
        &config,
        &backend,
    )
    .await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn rejects_when_tokens_exhausted() {
    let mut config = Config::default();
    config.rate_limit.max_tokens = 2.0;
    config.rate_limit.refill_rate = 0.0;

    let route = RouteConfig { token_weight: 1.0, ..RouteConfig::default() };
    let backend = rate_limit::backend::DynBackend::InMemory(Cache::new(100));
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    assert!(
        rate_limit::limiter::run(
            &route,
            rate_limit::key::TokenBucketKey::Ip(ip),
            &config,
            &backend,
        )
        .await
        .is_ok()
    );
    assert!(
        rate_limit::limiter::run(
            &route,
            rate_limit::key::TokenBucketKey::Ip(ip),
            &config,
            &backend,
        )
        .await
        .is_ok()
    );
    assert!(
        rate_limit::limiter::run(
            &route,
            rate_limit::key::TokenBucketKey::Ip(ip),
            &config,
            &backend,
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn different_ips_get_separate_buckets() {
    let mut config = Config::default();
    config.rate_limit.max_tokens = 1.0;
    config.rate_limit.refill_rate = 0.0;

    let route = RouteConfig { token_weight: 1.0, ..RouteConfig::default() };
    let backend = rate_limit::backend::DynBackend::InMemory(Cache::new(100));

    let ip1 = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip2 = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));

    assert!(
        rate_limit::limiter::run(
            &route,
            rate_limit::key::TokenBucketKey::Ip(ip1),
            &config,
            &backend,
        )
        .await
        .is_ok()
    );
    assert!(
        rate_limit::limiter::run(
            &route,
            rate_limit::key::TokenBucketKey::Ip(ip1),
            &config,
            &backend,
        )
        .await
        .is_err()
    );

    assert!(
        rate_limit::limiter::run(
            &route,
            rate_limit::key::TokenBucketKey::Ip(ip2),
            &config,
            &backend,
        )
        .await
        .is_ok()
    );
    assert!(
        rate_limit::limiter::run(
            &route,
            rate_limit::key::TokenBucketKey::Ip(ip2),
            &config,
            &backend,
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn high_weight_drains_faster() {
    let mut config = Config::default();
    config.rate_limit.max_tokens = 10.0;
    config.rate_limit.refill_rate = 0.0;

    let route = RouteConfig { token_weight: 5.0, ..RouteConfig::default() };
    let backend = rate_limit::backend::DynBackend::InMemory(Cache::new(100));
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    assert!(
        rate_limit::limiter::run(
            &route,
            rate_limit::key::TokenBucketKey::Ip(ip),
            &config,
            &backend,
        )
        .await
        .is_ok()
    );
    assert!(
        rate_limit::limiter::run(
            &route,
            rate_limit::key::TokenBucketKey::Ip(ip),
            &config,
            &backend,
        )
        .await
        .is_ok()
    );
    assert!(
        rate_limit::limiter::run(
            &route,
            rate_limit::key::TokenBucketKey::Ip(ip),
            &config,
            &backend,
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn tokens_refill_after_time() {
    let mut config = Config::default();
    config.rate_limit.max_tokens = 1.0;
    config.rate_limit.refill_rate = 10.0;

    let route = RouteConfig { token_weight: 1.0, ..RouteConfig::default() };
    let backend = rate_limit::backend::DynBackend::InMemory(Cache::new(100));
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    assert!(
        rate_limit::limiter::run(
            &route,
            rate_limit::key::TokenBucketKey::Ip(ip),
            &config,
            &backend,
        )
        .await
        .is_ok()
    );
    assert!(
        rate_limit::limiter::run(
            &route,
            rate_limit::key::TokenBucketKey::Ip(ip),
            &config,
            &backend,
        )
        .await
        .is_err()
    );

    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    assert!(
        rate_limit::limiter::run(
            &route,
            rate_limit::key::TokenBucketKey::Ip(ip),
            &config,
            &backend,
        )
        .await
        .is_ok()
    );
}
