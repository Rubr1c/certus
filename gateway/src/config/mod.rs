pub mod cfg_utils;
pub mod error;

use std::collections::HashMap;

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::server::upstream::HttpVersion;

//TODO: some config options live duplicated in memory
//      in 2 seperate places should probably optimize that
//
//      make the config be able to be read from db instead of
//      yaml if no file is found

/// Command line argument parser with all the commands
/// available in certus
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CmdArgs {
    #[arg(short, long)]
    pub config: Option<String>,
    #[arg(long)]
    pub save: bool,
}

/// Config struct for all configurable server options
#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
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
    #[serde(default = "default_auth_prefix")]
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
    pub http_version: HttpVersion,
    #[serde(default = "default_max_connections")]
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
    #[serde(default = "default_max_tokens")]
    pub max_tokens: f64,
    #[serde(default = "default_refill_rate")]
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
    #[serde(default = "default_cache_size")]
    pub size: u64,
    #[serde(default, rename = "type")]
    pub cache_type: StorageType,
    pub ttl: Option<u64>,
    pub tti: Option<u64>,
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
    #[serde(default = "default_server_addr")]
    pub default_server: String,
}

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
            http_version: HttpVersion::HTTP1,
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

fn default_port() -> u16 {
    8080
}

fn default_server_addr() -> String {
    "127.0.0.1:80".to_string()
}

fn default_max_connections() -> usize {
    100
}

fn default_max_tokens() -> f64 {
    100.0
}

fn default_refill_rate() -> f64 {
    1.0
}

fn default_auth_prefix() -> String {
    "Bearer".to_string()
}

fn default_cache_size() -> u64 {
    1000
}
