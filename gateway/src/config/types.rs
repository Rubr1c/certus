use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::upstream::protocol;

use super::defaults;

//TODO: some config options live duplicated in memory
//      in 2 seperate places should probably optimize that
//
//      make the config be able to be read from db instead of
//      yaml if no file is found

/// Config struct for all configurable server options
#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "defaults::default_port")]
    pub port: u16,
    #[serde(default)]
    pub origins: Vec<String>,
}

/// Enum for all authentication types available
#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    #[default]
    None,
    JWT {
        secret: String,
        #[serde(default)]
        algorithm: jsonwebtoken::Algorithm,
    },
}

/// Config struct that just holds the method [`AuthType`]
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthConfig {
    pub method: AuthType,
    #[serde(default = "defaults::default_auth_prefix")]
    pub prefix: String,
}

/// Config struct for tls to run in https
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TLSConfig {
    pub cert_path: String,
    pub key_path: String,
}

/// Config struct that holds all configurable options
/// for each route registered in the config
#[derive(Debug, Deserialize, Serialize)]
pub struct RouteConfig {
    pub endpoints: Vec<String>,
    #[serde(default)]
    pub is_static: bool,
    #[serde(default)]
    pub needs_auth: bool,
    #[serde(default)]
    pub http_version: protocol::HttpVersion,
    #[serde(default = "defaults::default_max_connections")]
    pub max_connections: usize,
    #[serde(default)]
    pub token_weight: f64,
    #[serde(default)]
    pub no_cache: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RateLimitKey {
    #[default]
    Ip,
    Token,
    Header(String),
}

/// Config struct that holds all rate limiting options
#[derive(Debug, Deserialize, Serialize)]
pub struct RateLimitConfig {
    #[serde(default = "defaults::default_max_tokens")]
    pub max_tokens: f64,
    #[serde(default = "defaults::default_refill_rate")]
    pub refill_rate: f64,
    #[serde(default)]
    pub key: RateLimitKey,
    #[serde(default, rename = "type")]
    pub rl_type: StorageType,
}

/// Config struct that holds all connection options
#[derive(Debug, Deserialize, Serialize)]
pub struct ConnectionConfig {
    pub connect_timeout: u64,
}

/// Enum for all storage types available
#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    #[default]
    #[serde(rename = "in_memory")]
    InMemory,
    Redis {
        url: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StaticCacheConfig {
    #[serde(default, rename = "type")]
    pub cache_type: StorageType,
}

//THIS IS SO BAD PLEASE CHANGE

/// Config struct that holds all cache options
#[derive(Debug, Deserialize, Serialize)]
pub struct CacheConfig {
    #[serde(default = "defaults::default_cache_size")]
    pub size: u64,
    #[serde(default, rename = "type")]
    pub cache_type: StorageType,
    pub ttl: Option<u64>,
    pub tti: Option<u64>,
    #[serde(default = "defaults::default_cache_max_size")]
    pub max_size: u64,
    #[serde(default, rename = "static")]
    pub static_cache: StaticCacheConfig,
}

/// Main config struct that holds all configurable items
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub routes: HashMap<String, RouteConfig>,
    pub tls: Option<TLSConfig>,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    #[serde(default)]
    pub connection: ConnectionConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default = "defaults::default_server_addr")]
    pub default_server: String,
}
