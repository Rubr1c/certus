use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    db::repository::{log, metrics, schema},
    logging::LogEntryDTO,
    metrics::MetricEvent,
    schema::ReqResSchemaDTO,
};

/// Saves a batch of logs concurrently
///
/// # Arguments
///
/// * `conn` - arc mutex connection of a sqlite database
/// * `batch` - mutable vector of logs to save
pub fn log(
    conn: &Arc<Mutex<rusqlite::Connection>>,
    batch: &mut Vec<LogEntryDTO>,
) {
    let logs = std::mem::replace(batch, Vec::with_capacity(100));
    let conn_clone = Arc::clone(conn);

    tokio::task::spawn_blocking(move || {
        let mut conn_guard = conn_clone.lock();

        if let Err(e) = log::save(&mut conn_guard, logs) {
            tracing::error!(err = ?e, "Failed to batch save logs");
        }
    });
}

pub fn metrics(
    conn: &Arc<Mutex<rusqlite::Connection>>,
    batch: &mut Vec<MetricEvent>,
) {
    let metrics = std::mem::replace(batch, Vec::with_capacity(100));
    let conn_clone = Arc::clone(conn);

    tokio::task::spawn_blocking(move || {
        let mut conn_guard = conn_clone.lock();

        if let Err(e) = metrics::write::save(&mut conn_guard, metrics) {
            tracing::error!(err = ?e, "Failed to batch save metrics");
        }
    });
}

pub fn schema(
    conn: &Arc<Mutex<rusqlite::Connection>>,
    batch: &mut Vec<ReqResSchemaDTO>,
) {
    let schemas = std::mem::replace(batch, Vec::with_capacity(100));
    let conn_clone = Arc::clone(conn);

    tokio::task::spawn_blocking(move || {
        let schemas = schemas.into_iter().map(|dto| dto.into_s()).collect();
        let mut conn_guard = conn_clone.lock();

        if let Err(e) = schema::save(&mut conn_guard, schemas) {
            tracing::error!(err = ?e, "Failed to batch save schemas");
        }
    });
}
