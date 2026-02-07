use std::collections::HashMap;
use std::net::SocketAddr;

use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CmdArgs {
    #[arg(short, long)]
    pub config: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub origins: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig { port: default_port(), origins: Vec::new() }
    }
}

fn default_port() -> u16 {
    8080
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    #[default]
    None,
    JWT {
        secret: String,
    },
}

#[derive(Debug, Default, Deserialize)]
pub struct AuthConfig {
    pub method: AuthType,
}

#[derive(Debug, Default, Deserialize)]
pub struct RouteConfig {
    pub endpoints: Vec<SocketAddr>,
    pub auth: Option<AuthConfig>,
    pub is_static: Option<bool>,
    pub needs_auth: Option<bool>,
    #[serde(default = "default_token_weight")]
    pub token_weight: f64,
}

#[derive(Debug, Default, Deserialize)]
pub struct RateLimitConfig {
    pub max_tokens: f64,
    pub refill_rate: f64,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: Option<AuthConfig>,
    #[serde(default = "default_rate_limit")]
    pub rate_limit: RateLimitConfig,
    pub routes: HashMap<String, RouteConfig>,
    #[serde(default = "default_socket_addr")]
    pub default_server: SocketAddr,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig::default(),
            auth: None,
            rate_limit: default_rate_limit(),
            routes: HashMap::new(),
            default_server: default_socket_addr(),
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
