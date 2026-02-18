pub mod cfg_utils;
pub mod error;

use std::{collections::HashMap, net::SocketAddr};

use clap::Parser;
use serde::Deserialize;

use crate::server::upstream::Protocol;

//TODO: change all optional with defaults to access them better
//      in the code i realized that it would be better that way
//      but im not focused on config right now. all new options
//      will do that.
//
//      also i can just implment Default myself for some of them
//      which would make the code more clean i think
//
//      some config options live duplicated in memory in 2 seperate
//      places should probably optimize that

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

impl Default for ServerConfig {
    /// Returns default server config with port being 8080
    /// and empty origins vec
    fn default() -> Self {
        ServerConfig { port: default_port(), origins: Vec::new() }
    }
}

fn default_port() -> u16 {
    8080
}

/// Enum for all authentication types available
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    #[default]
    None,
    JWT {
        secret: String,
    },
}

/// Config struct that just holds the method [`AuthType`]
#[derive(Debug, Default, Deserialize)]
pub struct AuthConfig {
    pub method: AuthType,
}

/// Config struct that holds all configurable options
/// for each route registered in the config
#[derive(Debug, Default, Deserialize)]
pub struct RouteConfig {
    pub endpoints: Vec<SocketAddr>,
    pub is_static: Option<bool>,
    pub needs_auth: Option<bool>,
    #[serde(default)]
    pub protocol: Protocol,
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    #[serde(default = "default_token_weight")]
    pub token_weight: f64,
}

/// Config struct that holds all rate limiting options
#[derive(Debug, Default, Deserialize)]
pub struct RateLimitConfig {
    pub max_tokens: f64,
    pub refill_rate: f64,
}

/// Config struct that holds all connection options
#[derive(Debug, Default, Deserialize)]
pub struct ConnectionConfig {
    pub connect_timeout: u64,
}

/// Config struct that holds all cache options
#[derive(Debug, Default, Deserialize)]
pub struct CacheConfig {
    pub size: u64,
}

/// Main config struct that holds all configurable items
#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: Option<AuthConfig>,
    #[serde(default = "default_rate_limit")]
    pub rate_limit: RateLimitConfig,
    pub routes: HashMap<String, RouteConfig>,
    #[serde(default = "default_connetion_config")]
    pub connection: ConnectionConfig,
    #[serde(default = "default_cache_config")]
    pub cache: CacheConfig,
    #[serde(default = "default_socket_addr")]
    pub default_server: SocketAddr,
}

impl Default for Config {
    /// Returns default config for certus (not recommended)
    fn default() -> Self {
        Config {
            server: ServerConfig::default(),
            auth: None,
            rate_limit: default_rate_limit(),
            routes: HashMap::new(),
            default_server: default_socket_addr(),
            connection: default_connetion_config(),
            cache: default_cache_config(),
        }
    }
}

fn default_socket_addr() -> SocketAddr {
    "127.0.0.1:80".parse().unwrap()
}

fn default_token_weight() -> f64 {
    1.0
}

fn default_rate_limit() -> RateLimitConfig {
    RateLimitConfig { max_tokens: 100.0, refill_rate: 1.0 }
}

fn default_connetion_config() -> ConnectionConfig {
    ConnectionConfig { connect_timeout: 2000 }
}

fn default_max_connections() -> usize {
    100
}

fn default_cache_config() -> CacheConfig {
    CacheConfig { size: 1000 }
}
