pub mod cfg_utils;
pub mod error;

use std::{collections::HashMap, net::SocketAddr};

use clap::Parser;
use serde::Deserialize;

use crate::server::upstream::Protocol;

//TODO: some config options live duplicated in memory
//      in 2 seperate places should probably optimize that

/// Command line argument parser with all the commands
/// available in certus
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CmdArgs {
    #[arg(short, long)]
    pub config: Option<String>,
}

/// Config struct for all configurable server options
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub origins: Vec<String>,
}

/// Enum for all authentication types available
#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    #[default]
    None,
    JWT {
        secret: String,
    },
}

/// Config struct that just holds the method [`AuthType`]
#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub method: AuthType,
}

/// Config struct that holds all configurable options
/// for each route registered in the config
#[derive(Debug, Deserialize)]
pub struct RouteConfig {
    pub endpoints: Vec<SocketAddr>,
    #[serde(default)]
    pub is_static: bool,
    #[serde(default)]
    pub needs_auth: bool,
    #[serde(default)]
    pub protocol: Protocol,
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    #[serde(default)]
    pub token_weight: f64,
}

/// Config struct that holds all rate limiting options
#[derive(Debug, Deserialize)]
pub struct RateLimitConfig {
    pub max_tokens: f64,
    #[serde(default = "default_refill_rate")]
    pub refill_rate: f64,
}

/// Config struct that holds all connection options
#[derive(Debug, Deserialize)]
pub struct ConnectionConfig {
    pub connect_timeout: u64,
}

/// Config struct that holds all cache options
#[derive(Debug, Deserialize)]
pub struct CacheConfig {
    pub size: u64,
}

/// Main config struct that holds all configurable items
#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub routes: HashMap<String, RouteConfig>,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    #[serde(default)]
    pub connection: ConnectionConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default = "default_socket_addr")]
    pub default_server: SocketAddr,
}

impl Default for Config {
    /// Default config for certus (not recommended)
    fn default() -> Self {
        Config {
            server: ServerConfig::default(),
            auth: AuthConfig::default(),
            rate_limit: RateLimitConfig::default(),
            routes: HashMap::new(),
            default_server: default_socket_addr(),
            connection: ConnectionConfig::default(),
            cache: CacheConfig::default(),
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
            protocol: Protocol::HTTP1,
            max_connections: 100,
            token_weight: 0.0,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        RateLimitConfig { max_tokens: 100.0, refill_rate: 1.0 }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig { size: 1000 }
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        ConnectionConfig { connect_timeout: 2000 }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig { method: AuthType::default() }
    }
}

fn default_port() -> u16 {
    8080
}

fn default_socket_addr() -> SocketAddr {
    "127.0.0.1:80".parse().unwrap()
}

fn default_max_connections() -> usize {
    100
}

fn default_refill_rate() -> f64 {
    1.0
}
