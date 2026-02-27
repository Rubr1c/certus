use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::Connection;
use tokio::sync::mpsc;

use crate::metrics::MetricEvent;

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

fn test_db_conn() -> Arc<Mutex<Connection>> {
    Arc::new(Mutex::new(Connection::open_in_memory().unwrap()))
}

fn test_metrics_tx() -> mpsc::Sender<MetricEvent> {
    let (tx, _rx) = mpsc::channel::<MetricEvent>(16);
    tx
}
