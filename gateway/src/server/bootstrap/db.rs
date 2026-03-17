use std::sync::Arc;

use parking_lot::Mutex;

use crate::db;

#[inline]
pub fn run() -> (
    Arc<Mutex<rusqlite::Connection>>,
    Arc<Mutex<rusqlite::Connection>>,
    Arc<Mutex<rusqlite::Connection>>,
) {
    let log_conn = db::connection::connect_db().expect("log db");
    let metrics_conn = db::connection::connect_db().expect("metrics db");
    let schema_conn = db::connection::connect_db().expect("schema db");

    db::migration::migrate(&log_conn).expect("migrate");

    let log_conn = Arc::new(Mutex::new(log_conn));
    let metrics_conn = Arc::new(Mutex::new(metrics_conn));
    let schema_conn = Arc::new(Mutex::new(schema_conn));

    (log_conn, metrics_conn, schema_conn)
}
