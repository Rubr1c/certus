//! Snapshot-style checks so the dashboard TypeScript types stay aligned with serde JSON.

use crate::config::{
    AuthConfig, AuthType, CacheConfig, Config, RateLimitConfig, RateLimitKey,
    StaticCacheConfig, StorageType,
};

#[test]
fn auth_type_none_round_trip_json() {
    let m = AuthType::None;
    // Unit variants serialize as a bare JSON string (dashboard `AuthType | "none"`).
    assert_eq!(serde_json::to_string(&m).unwrap(), r#""none""#);
    let jwt = AuthType::JWT {
        secret: "s".into(),
        algorithm: jsonwebtoken::Algorithm::HS256,
    };
    assert_eq!(
        serde_json::to_string(&jwt).unwrap(),
        r#"{"jwt":{"secret":"s","algorithm":"HS256"}}"#
    );
}

#[test]
fn rate_limit_ip_token_header_json() {
    assert_eq!(serde_json::to_string(&RateLimitKey::Ip).unwrap(), r#""ip""#);
    assert_eq!(
        serde_json::to_string(&RateLimitKey::Token).unwrap(),
        r#""token""#
    );
    assert_eq!(
        serde_json::to_string(&RateLimitKey::Header("X-Api-Key".into()))
            .unwrap(),
        r#"{"header":"X-Api-Key"}"#
    );
}

#[test]
fn storage_type_in_memory_and_redis_json() {
    assert_eq!(
        serde_json::to_string(&StorageType::InMemory).unwrap(),
        r#""in_memory""#
    );
    assert_eq!(
        serde_json::to_string(&StorageType::Redis {
            url: "redis://127.0.0.1:6379".into()
        })
        .unwrap(),
        r#"{"redis":{"url":"redis://127.0.0.1:6379"}}"#
    );
}

#[test]
fn full_config_default_serializes() {
    let c = Config::default();
    let v: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    assert!(v.get("cache").is_some());
    assert!(v.get("auth").is_some());
}

#[test]
fn jwt_auth_round_trip() {
    let auth = AuthConfig {
        method: AuthType::JWT {
            secret: "s".into(),
            algorithm: jsonwebtoken::Algorithm::HS256,
        },
        prefix: "Bearer".into(),
    };
    let json = serde_json::to_string(&auth).unwrap();
    let back: AuthConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(auth.prefix, back.prefix);
    match back.method {
        AuthType::JWT { secret, .. } => assert_eq!(secret, "s"),
        _ => panic!("expected jwt"),
    }
}

#[test]
fn cache_config_round_trip() {
    let cache = CacheConfig {
        size: 1,
        cache_type: StorageType::InMemory,
        ttl: Some(300),
        tti: None,
        max_size: 100,
        static_cache: StaticCacheConfig { cache_type: StorageType::InMemory },
    };
    let json = serde_json::to_string(&cache).unwrap();
    let back: CacheConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.size, cache.size);
    assert_eq!(back.ttl, cache.ttl);
}

#[test]
fn rate_limit_config_round_trip() {
    let rl = RateLimitConfig {
        max_tokens: 10.0,
        refill_rate: 1.0,
        key: RateLimitKey::Header("h".into()),
        rl_type: StorageType::InMemory,
    };
    let json = serde_json::to_string(&rl).unwrap();
    let back: RateLimitConfig = serde_json::from_str(&json).unwrap();
    assert!((back.max_tokens - 10.0).abs() < f64::EPSILON);
}
