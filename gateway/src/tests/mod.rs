use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::{broadcast, mpsc};

use crate::{cli, logging, metrics, schema};

pub mod auth;
pub mod cache;
pub mod connection;
pub mod lb;
pub mod pipeline;
pub mod rate_limit;
pub mod reload;
pub mod router;

fn create_addrs(count: i32) -> Vec<String> {
    let mut addrs = Vec::<String>::new();

    for i in 0..count {
        addrs.push(format!("127.0.0.{}:3000", i));
    }

    addrs
}

fn test_db_conn() -> Arc<Mutex<rusqlite::Connection>> {
    Arc::new(Mutex::new(rusqlite::Connection::open_in_memory().unwrap()))
}

fn test_metrics_tx() -> mpsc::Sender<metrics::types::MetricEvent> {
    let (tx, _rx) = mpsc::channel::<metrics::types::MetricEvent>(16);
    tx
}

fn test_schema_tx() -> mpsc::Sender<schema::types::ReqResSchemaDTO> {
    let (tx, _rx) = mpsc::channel::<schema::types::ReqResSchemaDTO>(16);
    tx
}

fn test_log_tx() -> Option<broadcast::Sender<logging::types::LogEntryDTO>> {
    None
}

fn test_args() -> Arc<cli::CmdArgs> {
    Arc::new(cli::CmdArgs {
        config: String::new(),
        save: false,
        ws: Vec::new(),
    })
}
