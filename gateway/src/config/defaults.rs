use std::collections::HashMap;

use crate::upstream::protocol;

use super::types::{
    AuthConfig, AuthType, CacheConfig, Config, ConnectionConfig,
    RateLimitConfig, RateLimitKey, RouteConfig, ServerConfig,
    StaticCacheConfig, StorageType,
};

impl Default for Config {
    /// Default config for certus (not recommended)
    fn default() -> Self {
        Config {
            server: ServerConfig::default(),
            auth: AuthConfig::default(),
            rate_limit: RateLimitConfig::default(),
            routes: HashMap::new(),
            default_server: default_server_addr(),
            connection: ConnectionConfig::default(),
            cache: CacheConfig::default(),
            tls: None,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig { port: default_port(), origins: Vec::new() }
    }
}

impl Default for RouteConfig {
    fn default() -> Self {
        RouteConfig {
            endpoints: Vec::new(),
            is_static: false,
            needs_auth: false,
            http_version: protocol::HttpVersion::HTTP1,
            max_connections: 100,
            token_weight: 0.0,
            no_cache: false,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        RateLimitConfig {
            max_tokens: default_max_tokens(),
            refill_rate: default_refill_rate(),
            key: RateLimitKey::default(),
            rl_type: StorageType::default(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            size: 1000,
            ttl: None,
            tti: None,
            max_size: default_cache_max_size(),
            cache_type: StorageType::default(),
            static_cache: StaticCacheConfig::default(),
        }
    }
}

impl Default for StaticCacheConfig {
    fn default() -> Self {
        StaticCacheConfig { cache_type: StorageType::default() }
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        ConnectionConfig { connect_timeout: 2000 }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            method: AuthType::default(),
            prefix: default_auth_prefix(),
        }
    }
}

pub fn default_port() -> u16 {
    8080
}

pub fn default_server_addr() -> String {
    "127.0.0.1:80".to_string()
}

pub fn default_max_connections() -> usize {
    100
}

pub fn default_max_tokens() -> f64 {
    100.0
}

pub fn default_refill_rate() -> f64 {
    1.0
}

pub fn default_auth_prefix() -> String {
    "Bearer".to_string()
}

pub fn default_cache_size() -> u64 {
    1000
}

pub fn default_cache_max_size() -> u64 {
    10_485_760
}
