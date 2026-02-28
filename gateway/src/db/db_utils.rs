use std::{sync::Arc, time::Duration};

use parking_lot::Mutex;
use rusqlite::Connection;

use crate::{
    config::Config,
    db::models::LogEntry,
    logging::log_util::LogEntryDTO,
    metrics::{CacheHitMetric, MetricEvent, RequestMetric},
};

pub fn connect_db() -> rusqlite::Result<Connection> {
    //TODO: Change path and name
    let conn = Connection::open("dev.db")?;

    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.busy_timeout(Duration::from_millis(5000))?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;

    Ok(conn)
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
        "CREATE TABLE IF NOT EXISTS config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            config_data TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );",
        "CREATE TABLE IF NOT EXISTS request_metrics(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            route TEXT NOT NULL,
            status_code INTEGER NOT NULL,
            duration INTEGER NOT NULL
        )",
        "CREATE TABLE IF NOT EXISTS cache_hit_metrics(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            route TEXT NOT NULL
        )",
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
    let mut stmt =
        conn.prepare("SELECT id, timestamp, level, message, fields FROM logs")?;

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

/// Saves gateway config to sqlite database
///
/// # Arguments
///
/// * `conn` - connection to sqlite database
/// * `config` - config struct to save
///
/// # Errors
///
/// Returns an error if:
/// * Failed to execute query
///
/// # Panics
///
/// Panics if:
/// * Failed to parse config to json
pub fn save_config(conn: &Connection, config: &Config) -> rusqlite::Result<()> {
    let json_str = serde_json::to_string(config)
        .expect("failed to serialize config to JSON");

    conn.execute(
        "INSERT INTO config (id, config_data, updated_at)
             VALUES (1, ?1, CURRENT_TIMESTAMP)
                ON CONFLICT(id) DO UPDATE SET 
                config_data = excluded.config_data,
                updated_at = CURRENT_TIMESTAMP
            ",
        [json_str],
    )?;

    Ok(())
}

/// Gets config from sqlite database
///
/// # Arguments
///
/// * `conn` - connection to sqlite database
///
/// # Errors
///
/// Returns an error if:
/// * Failed to get config row
/// * Failed to parse json to config
pub fn get_config(conn: &Connection) -> rusqlite::Result<Config> {
    conn.query_row("SELECT config_data FROM config WHERE id = 1", [], |row| {
        let json_str: String = row.get(0)?;
        serde_json::from_str(&json_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            )
        })
    })
}

pub fn save_req_metric(
    conn: &Connection,
    metric: RequestMetric,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO request_metrics (route, timestamp, status_code, duration) 
                  VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            metric.route,
            metric.timestamp.to_string(),
            metric.status_code,
            metric.duration_ms as i64
        ],
    )?;

    Ok(())
}

pub fn save_req_metrics(
    conn: &mut Connection,
    metrics: Vec<&RequestMetric>,
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    {
        let mut query = tx.prepare(
           "INSERT INTO request_metrics (route, timestamp, status_code, duration) 
                  VALUES (?1, ?2, ?3, ?4)",
        )?;

        for metric in metrics {
            query.execute(rusqlite::params![
                metric.route,
                metric.timestamp.to_string(),
                metric.status_code,
                metric.duration_ms as i64
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn save_cache_hit_metrics(
    conn: &mut Connection,
    metrics: Vec<&CacheHitMetric>,
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    {
        let mut query = tx.prepare(
            "INSERT INTO cache_hit_metrics (route, timestamp)
                  VALUES (?1, ?2)",
        )?;

        for metric in metrics {
            query.execute(rusqlite::params![
                metric.route,
                metric.timestamp.to_string(),
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn save_metrics(
    conn: &mut Connection,
    metrics: Vec<MetricEvent>,
) -> rusqlite::Result<()> {
    let req_metrics = metrics
        .iter()
        .filter_map(|metric| match metric {
            MetricEvent::Request(m) => Some(m),
            _ => None,
        })
        .collect::<Vec<_>>();

    let cache_hit_metrics = metrics
        .iter()
        .filter_map(|metric| match metric {
            MetricEvent::CacheHit(m) => Some(m),
            _ => None,
        })
        .collect::<Vec<_>>();

    save_req_metrics(conn, req_metrics)?;
    save_cache_hit_metrics(conn, cache_hit_metrics)?;

    Ok(())
}

// should prob move this into another mod

/// Saves a batch of logs concurrently
///
/// # Arguments
///
/// * `conn` - arc mutex connection of a sqlite database
/// * `batch` - mutable vector of logs to save
pub async fn flush_log_batch(
    conn: &Arc<Mutex<Connection>>,
    batch: &mut Vec<LogEntryDTO>,
) {
    let logs = std::mem::replace(batch, Vec::with_capacity(100));
    let conn_clone = Arc::clone(conn);

    tokio::task::spawn_blocking(move || {
        let mut conn_guard = conn_clone.lock();

        if let Err(e) = save_logs(&mut conn_guard, logs) {
            tracing::error!(err = ?e, "Failed to batch save logs");
        }
    });
}

pub async fn flush_metric_batch(
    conn: &Arc<Mutex<Connection>>,
    batch: &mut Vec<MetricEvent>,
) {
    let metrics = std::mem::replace(batch, Vec::with_capacity(100));
    let conn_clone = Arc::clone(conn);

    tokio::task::spawn_blocking(move || {
        let mut conn_guard = conn_clone.lock();

        if let Err(e) = save_metrics(&mut conn_guard, metrics) {
            tracing::error!(err = ?e, "Failed to batch save logs");
        }
    });
}
