use clap::{Parser, ValueEnum};

use serde::Serialize;

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize,
)]
pub enum WebSocketType {
    Logs,
    Metrics,
}

/// Certus
#[derive(Parser, Debug, Serialize)]
#[command(version, about, long_about = None)]
pub struct CmdArgs {
    /// YAML config path for certus
    #[arg(short, long, default_value_t = String::from("certus.config.yaml"))]
    pub config: String,

    /// Save config to sqlite db
    #[arg(long)]
    pub save: bool,

    /// Specifiy what web socket server to expose for real time updates
    #[arg(long, value_enum, value_delimiter = ',')]
    pub ws: Vec<WebSocketType>,
}
