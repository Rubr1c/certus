use std::{sync::Arc, time::Duration};

use parking_lot::Mutex;
use rusqlite::Connection;

use crate::{
    config::Config,
    db::models::{LogEntry, ReqResSchema, ReqResSchemaDTO},
    logging::log_util::LogEntryDTO,
    metrics::{CacheMetric, CacheResult, MetricEvent, RequestMetric},
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
            target TEXT NOT NULL,
            message TEXT NOT NULL,
            fields TEXT NOT NULL
        );",
        "CREATE TABLE IF NOT EXISTS config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            config_data TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );",
        "CREATE TABLE IF NOT EXISTS request_metrics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            route TEXT NOT NULL,
            status_code INTEGER NOT NULL,
            duration_total_ms INTEGER NOT NULL,
            duration_upstream_ms INTEGER NOT NULL,
            bytes_in INTEGER NOT NULL,
            bytes_out INTEGER NOT NULL,
            client_ip TEXT NOT NULL,
            method TEXT NOT NULL,
            upstream_addr TEXT
        )",
        "CREATE TABLE IF NOT EXISTS cache_metrics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            route TEXT NOT NULL,
            result TEXT NOT NULL
        )",
        "CREATE TABLE IF NOT EXISTS req_res_schemas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            route TEXT NOT NULL,
            req_headers TEXT NOT NULL,
            res_headers TEXT NOT NULL
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
pub fn save_log(conn: &Connection, entry: LogEntryDTO) -> rusqlite::Result<()> {
    let fields_json = serde_json::to_string(&entry.fields)
        .unwrap_or_else(|_| "{}".to_string());

    conn.execute(
        "INSERT INTO logs (timestamp, level, target, message, fields) 
                  VALUES (?1, ?2, ?3, ?4, ?5)",
        [
            entry.timestamp,
            entry.level,
            entry.target,
            entry.message,
            fields_json,
        ],
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
            "INSERT INTO logs (timestamp, level, target, message, fields)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;

        for entry in entries {
            let fields_json = serde_json::to_string(&entry.fields)
                .unwrap_or_else(|_| "{}".to_string());

            query.execute([
                entry.timestamp,
                entry.level,
                entry.target,
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
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, level, target, message, fields FROM logs",
    )?;

    let logs = stmt.query_map([], |row| {
        Ok(LogEntry {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            level: row.get(2)?,
            target: row.get(3)?,
            message: row.get(4)?,
            fields: row.get(5)?,
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
        "INSERT INTO request_metrics (
            route,
            timestamp,
            status_code,
            duration_total_ms,
            duration_upstream_ms,
            bytes_in,
            bytes_out,
            client_ip,
            method,
            upstream_addr
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            metric.route.as_ref(),
            metric.timestamp.to_string(),
            metric.status_code,
            metric.duration_total_ms as i64,
            metric.duration_upstream_ms as i64,
            metric.bytes_in as i64,
            metric.bytes_out as i64,
            metric.client_ip.to_string(),
            metric.method.to_string(),
            metric.upstream_addr.as_deref()
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
            "INSERT INTO request_metrics (
                route,
                timestamp,
                status_code,
                duration_total_ms,
                duration_upstream_ms,
                bytes_in,
                bytes_out,
                client_ip,
                method,
                upstream_addr
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )?;

        for metric in metrics {
            query.execute(rusqlite::params![
                metric.route.as_ref(),
                metric.timestamp.to_string(),
                metric.status_code,
                metric.duration_total_ms as i64,
                metric.duration_upstream_ms as i64,
                metric.bytes_in as i64,
                metric.bytes_out as i64,
                metric.client_ip.to_string(),
                metric.method.to_string(),
                metric.upstream_addr.as_deref()
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn save_cache_metrics(
    conn: &mut Connection,
    metrics: Vec<&CacheMetric>,
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    {
        let mut query = tx.prepare(
            "INSERT INTO cache_metrics (route, timestamp, result)
                  VALUES (?1, ?2, ?3)",
        )?;

        for metric in metrics {
            let result = match metric.result {
                CacheResult::Hit => "hit",
                CacheResult::Miss => "miss",
                CacheResult::Bypass => "bypass",
            };
            query.execute(rusqlite::params![
                metric.route.as_ref(),
                metric.timestamp.to_string(),
                result,
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

    let cache_metrics = metrics
        .iter()
        .filter_map(|metric| match metric {
            MetricEvent::Cache(m) => Some(m),
            _ => None,
        })
        .collect::<Vec<_>>();

    save_req_metrics(conn, req_metrics)?;
    save_cache_metrics(conn, cache_metrics)?;

    Ok(())
}

pub fn save_req_res_schemas(
    conn: &mut Connection,
    req_res_schemas: Vec<ReqResSchema>,
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    {
        let mut query = tx.prepare(
            "INSERT INTO req_res_schemas (route, req_headers, res_headers)
             VALUES (?1, ?2, ?3)",
        )?;

        for schema in req_res_schemas {
            query.execute([
                schema.route,
                schema.req_headers,
                schema.res_headers,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// should prob move this into another mod

/// Saves a batch of logs concurrently
///
/// # Arguments
///
/// * `conn` - arc mutex connection of a sqlite database
/// * `batch` - mutable vector of logs to save
pub fn flush_log_batch(
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

pub fn flush_metric_batch(
    conn: &Arc<Mutex<Connection>>,
    batch: &mut Vec<MetricEvent>,
) {
    let metrics = std::mem::replace(batch, Vec::with_capacity(100));
    let conn_clone = Arc::clone(conn);

    tokio::task::spawn_blocking(move || {
        let mut conn_guard = conn_clone.lock();

        if let Err(e) = save_metrics(&mut conn_guard, metrics) {
            tracing::error!(err = ?e, "Failed to batch save metrics");
        }
    });
}

pub fn flush_req_res_schema_batch(
    conn: &Arc<Mutex<Connection>>,
    batch: &mut Vec<ReqResSchemaDTO>,
) {
    let schemas = std::mem::replace(batch, Vec::with_capacity(100));
    let conn_clone = Arc::clone(conn);

    tokio::task::spawn_blocking(move || {
        let schemas = schemas.into_iter().map(|dto| dto.into_s()).collect();
        let mut conn_guard = conn_clone.lock();

        if let Err(e) = save_req_res_schemas(&mut conn_guard, schemas) {
            tracing::error!(err = ?e, "Failed to batch save schemas");
        }
    });
}
