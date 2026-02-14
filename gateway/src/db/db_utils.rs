use std::sync::Arc;

use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::{db::models::LogEntry, logging::log_util::LogEntryDTO};

pub fn connect_db() -> Result<Connection, rusqlite::Error> {
    //TODO: Change path and name
    Connection::open("dev.db")
}

/// Runs all queries to create tables and initalize db
///
/// # Arguments
///
/// * `conn` - connection to sqlite database
///
/// # Errors
///
/// Returns an error if:
/// * query is failed to execute
pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let querys = vec![
        "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            level TEXT NOT NULL,
            message TEXT NOT NULL,
            fields TEXT NOT NULL
        );",
    ];

    for query in querys {
        conn.execute(query, ())?;
    }
    Ok(())
}

/// Saves a log to an sqlite database
///
/// # Arguments
///
/// * `conn` - connection to sqlite database
/// * `entry` - data of the log to save
///
/// # Errors
///
/// Returns an error if:
/// * Failed to execute query
///
pub fn save_log(conn: &Connection, entry: LogEntryDTO) -> rusqlite::Result<()> {
    let fields_json = serde_json::to_string(&entry.fields)
        .unwrap_or_else(|_| "{}".to_string());

    conn.execute(
        "INSERT INTO logs (timestamp, level, message, fields) 
                  VALUES (?1, ?2, ?3, ?4)",
        [entry.timestamp, entry.level, entry.message, fields_json],
    )?;

    Ok(())
}

/// Saves a vec of logs transactionaly
///
/// # Arguments
///
/// * `conn` - mutable connection to sqlite database
/// * `entries` - vector of logs to save
///
/// # Errors
///
/// Returns an error if:
/// * Failed to start transaction
/// * Failed to prepare query
/// * Failed to execute query
/// * Failed to commit transaction
pub fn save_logs(
    conn: &mut Connection,
    entries: Vec<LogEntryDTO>,
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    {
        let mut query = tx.prepare(
            "INSERT INTO logs (timestamp, level, message, fields)
         VALUES (?1, ?2, ?3, ?4)",
        )?;

        for entry in entries {
            let fields_json = serde_json::to_string(&entry.fields)
                .unwrap_or_else(|_| "{}".to_string());

            query.execute([
                entry.timestamp,
                entry.level,
                entry.message,
                fields_json,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Returns all logs from an sqlite database
///
/// # Arguments
///
/// * `conn` - connection to sqlite database
///
/// # Errors
///
/// Returns an error if:
/// * Failed to prepare query
/// * Failed to get element
/// * Failed to map query
pub fn get_logs(conn: &Connection) -> rusqlite::Result<Vec<LogEntry>> {
    let mut stmt = conn.prepare("SELECT id, entry FROM logs")?;
    let logs = stmt.query_map([], |row| {
        Ok(LogEntry {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            level: row.get(2)?,
            message: row.get(3)?,
            fields: row.get(4)?,
        })
    })?;
    let mut log_vec: Vec<LogEntry> = Vec::new();
    for log in logs {
        log_vec.push(log?);
    }
    Ok(log_vec)
}

// should prob move this into another mod

/// Saves a batch of logs concurrently
///
/// # Arguments
///
/// * `conn` - arc mutex connection of a sqlite database
/// * `batch` - mutable vector of logs to save
pub async fn flush_batch(
    conn: &Arc<Mutex<Connection>>,
    batch: &mut Vec<LogEntryDTO>,
) {
    let logs = std::mem::replace(batch, Vec::with_capacity(100));
    let conn_clone = Arc::clone(conn);

    tokio::task::spawn_blocking(move || {
        let mut conn_guard = conn_clone.blocking_lock();

        if let Err(e) = save_logs(&mut conn_guard, logs) {
            tracing::error!(err = ?e, "Failed to batch save logs");
        }
    });
}
