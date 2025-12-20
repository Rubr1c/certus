use std::collections::HashMap;

use clap::Parser;
use clap::command;
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
        expiration: u16,
    },
}

#[derive(Debug, Default, Deserialize)]
pub struct AuthConfig {
    pub method: AuthType,
    pub exculde: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RouteConfig {
    pub endpoints: Vec<String>,
    pub auth: Option<AuthConfig>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: Option<AuthConfig>,
    pub routes: HashMap<String, RouteConfig>,
    pub default_server: String,
}
