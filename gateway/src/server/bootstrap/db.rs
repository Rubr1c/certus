use std::sync::Arc;

use parking_lot::Mutex;

use crate::db;

#[inline]
pub fn run() -> (
    Arc<Mutex<rusqlite::Connection>>,
    Arc<Mutex<rusqlite::Connection>>,
    Arc<Mutex<rusqlite::Connection>>,
) {
    tracing::info!("Opening sqlite connections");
    let log_conn = match db::connection::connect_db() {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!(err = ?err, "Failed to open log database");
            panic!("failed to open log database");
        }
    };
    let metrics_conn = match db::connection::connect_db() {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!(err = ?err, "Failed to open metrics database");
            panic!("failed to open metrics database");
        }
    };
    let schema_conn = match db::connection::connect_db() {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!(err = ?err, "Failed to open schema database");
            panic!("failed to open schema database");
        }
    };

    if let Err(err) = db::migration::migrate(&log_conn) {
        tracing::error!(err = ?err, "Failed to apply sqlite migrations");
        panic!("failed to apply sqlite migrations");
    }
    tracing::info!("Sqlite connections ready and migrations applied");

    let log_conn = Arc::new(Mutex::new(log_conn));
    let metrics_conn = Arc::new(Mutex::new(metrics_conn));
    let schema_conn = Arc::new(Mutex::new(schema_conn));

    (log_conn, metrics_conn, schema_conn)
}
