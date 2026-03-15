use std::{sync::Arc, time::Duration};

use parking_lot::Mutex;
use rusqlite::Connection;

use crate::{
    config::Config,
    db::models::{
        CacheMetricBucket, CacheMetricRow, LogEntry, ReqResSchema,
        ReqResSchemaDTO, RequestMetricBucket, RequestMetricRow,
        RequestMetricSummary,
    },
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
            upstream_addr TEXT,
            early_exit TEXT
        )",
        "CREATE TABLE IF NOT EXISTS cache_metrics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            route TEXT NOT NULL,
            result TEXT NOT NULL
        )",
        "CREATE TABLE IF NOT EXISTS req_res_schemas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            full_path TEXT NOT NULL,
            method TEXT NOT NULL,
            query_params TEXT,
            status_code INTEGER NOT NULL,
            has_auth INTEGER NOT NULL DEFAULT 0,
            req_headers TEXT NOT NULL,
            res_headers TEXT NOT NULL
        )",
        "CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs (timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_request_metrics_timestamp ON request_metrics (timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_cache_metrics_timestamp ON cache_metrics (timestamp)",
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
            upstream_addr,
            early_exit
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
            metric.upstream_addr.as_deref(),
            metric.early_exit.as_ref().map(|e| e.as_str())
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
                upstream_addr,
                early_exit
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
                metric.upstream_addr.as_deref(),
                metric.early_exit.as_ref().map(|e| e.as_str())
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
            "INSERT INTO req_res_schemas (full_path, method, query_params, status_code, has_auth, req_headers, res_headers)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )?;

        for schema in req_res_schemas {
            query.execute(rusqlite::params![
                schema.full_path,
                schema.method,
                schema.query_params,
                schema.status_code,
                schema.has_auth,
                schema.req_headers,
                schema.res_headers,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn get_req_res_schemas(
    conn: &Connection,
    page: u32,
    page_size: u32,
) -> rusqlite::Result<Vec<ReqResSchema>> {
    let offset = page * page_size;

    let mut stmt = conn.prepare(
        "SELECT full_path, method, query_params, status_code, has_auth, req_headers, res_headers
         FROM req_res_schemas
         ORDER BY id DESC
         LIMIT ?1 OFFSET ?2",
    )?;

    let rows = stmt.query_map(rusqlite::params![page_size, offset], |row| {
        Ok(ReqResSchema {
            full_path: row.get(0)?,
            method: row.get(1)?,
            query_params: row.get(2)?,
            status_code: row.get(3)?,
            has_auth: row.get(4)?,
            req_headers: row.get(5)?,
            res_headers: row.get(6)?,
        })
    })?;

    rows.collect()
}

pub fn get_request_metrics(
    conn: &Connection,
    from: Option<&str>,
    to: Option<&str>,
    route: Option<&str>,
    status: Option<u16>,
    ip: Option<&str>,
    method: Option<&str>,
    page: u32,
    page_size: u32,
) -> rusqlite::Result<Vec<RequestMetricRow>> {
    let mut sql = String::from(
        "SELECT timestamp, route, status_code, duration_total_ms, duration_upstream_ms,
                bytes_in, bytes_out, client_ip, method, upstream_addr, early_exit
         FROM request_metrics",
    );

    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = from {
        conditions.push(format!("timestamp >= ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = to {
        conditions.push(format!("timestamp <= ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = route {
        conditions.push(format!("route = ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = status {
        conditions.push(format!("status_code = ?{}", params.len() + 1));
        params.push(Box::new(v));
    }
    if let Some(v) = ip {
        conditions.push(format!("client_ip = ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = method {
        conditions.push(format!("method = ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    let offset = page * page_size;
    sql.push_str(&format!(
        " ORDER BY timestamp DESC LIMIT ?{} OFFSET ?{}",
        params.len() + 1,
        params.len() + 2
    ));
    params.push(Box::new(page_size));
    params.push(Box::new(offset));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(RequestMetricRow {
            timestamp: row.get(0)?,
            route: row.get(1)?,
            status_code: row.get(2)?,
            duration_total_ms: row.get(3)?,
            duration_upstream_ms: row.get(4)?,
            bytes_in: row.get(5)?,
            bytes_out: row.get(6)?,
            client_ip: row.get(7)?,
            method: row.get(8)?,
            upstream_addr: row.get(9)?,
            early_exit: row.get(10)?,
        })
    })?;

    rows.collect()
}

pub fn get_cache_metrics(
    conn: &Connection,
    from: Option<&str>,
    to: Option<&str>,
    route: Option<&str>,
    page: u32,
    page_size: u32,
) -> rusqlite::Result<Vec<CacheMetricRow>> {
    let mut sql =
        String::from("SELECT timestamp, route, result FROM cache_metrics");

    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = from {
        conditions.push(format!("timestamp >= ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = to {
        conditions.push(format!("timestamp <= ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = route {
        conditions.push(format!("route = ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    let offset = page * page_size;
    sql.push_str(&format!(
        " ORDER BY timestamp DESC LIMIT ?{} OFFSET ?{}",
        params.len() + 1,
        params.len() + 2
    ));
    params.push(Box::new(page_size));
    params.push(Box::new(offset));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(CacheMetricRow {
            timestamp: row.get(0)?,
            route: row.get(1)?,
            result: row.get(2)?,
        })
    })?;

    rows.collect()
}

pub fn get_log_entries(
    conn: &Connection,
    from: Option<&str>,
    to: Option<&str>,
    level: Option<&str>,
    target: Option<&str>,
    search: Option<&str>,
    page: u32,
    page_size: u32,
) -> rusqlite::Result<Vec<LogEntry>> {
    let mut sql = String::from(
        "SELECT id, timestamp, level, target, message, fields FROM logs",
    );

    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = from {
        conditions.push(format!("timestamp >= ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = to {
        conditions.push(format!("timestamp <= ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = level {
        conditions.push(format!("level = ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = target {
        conditions.push(format!("target = ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = search {
        conditions.push(format!("message LIKE ?{}", params.len() + 1));
        params.push(Box::new(format!("%{}%", v)));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    let offset = page * page_size;
    sql.push_str(&format!(
        " ORDER BY timestamp DESC LIMIT ?{} OFFSET ?{}",
        params.len() + 1,
        params.len() + 2
    ));
    params.push(Box::new(page_size));
    params.push(Box::new(offset));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(LogEntry {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            level: row.get(2)?,
            target: row.get(3)?,
            message: row.get(4)?,
            fields: row.get(5)?,
        })
    })?;

    rows.collect()
}

pub fn get_request_metrics_aggregated(
    conn: &Connection,
    from: Option<&str>,
    to: Option<&str>,
    route: Option<&str>,
    status: Option<u16>,
    method: Option<&str>,
    interval_secs: i64,
) -> rusqlite::Result<Vec<RequestMetricBucket>> {
    let bucket_expr = format!(
        "strftime('%Y-%m-%dT%H:%M:%SZ', \
         (CAST(strftime('%s', REPLACE(timestamp, ' UTC', '')) AS INTEGER) / {0}) * {0}, \
         'unixepoch')",
        interval_secs
    );

    let mut sql = format!(
        "SELECT {bucket} AS bucket,
                COUNT(*) AS count,
                AVG(duration_total_ms) AS avg_duration_ms,
                MIN(duration_total_ms) AS min_duration_ms,
                MAX(duration_total_ms) AS max_duration_ms,
                AVG(duration_upstream_ms) AS avg_upstream_ms,
                SUM(bytes_in) AS bytes_in,
                SUM(bytes_out) AS bytes_out,
                SUM(CASE WHEN status_code >= 200 AND status_code < 300 THEN 1 ELSE 0 END) AS status_2xx,
                SUM(CASE WHEN status_code >= 300 AND status_code < 400 THEN 1 ELSE 0 END) AS status_3xx,
                SUM(CASE WHEN status_code >= 400 AND status_code < 500 THEN 1 ELSE 0 END) AS status_4xx,
                SUM(CASE WHEN status_code >= 500 THEN 1 ELSE 0 END) AS status_5xx,
                SUM(CASE WHEN early_exit IS NOT NULL THEN 1 ELSE 0 END) AS error_count
         FROM request_metrics",
        bucket = bucket_expr
    );

    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = from {
        conditions.push(format!("timestamp >= ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = to {
        conditions.push(format!("timestamp <= ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = route {
        conditions.push(format!("route = ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = status {
        conditions.push(format!("status_code = ?{}", params.len() + 1));
        params.push(Box::new(v));
    }
    if let Some(v) = method {
        conditions.push(format!("method = ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" GROUP BY bucket ORDER BY bucket ASC");

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(RequestMetricBucket {
            bucket: row.get(0)?,
            count: row.get(1)?,
            avg_duration_ms: row.get(2)?,
            min_duration_ms: row.get(3)?,
            max_duration_ms: row.get(4)?,
            avg_upstream_ms: row.get(5)?,
            bytes_in: row.get(6)?,
            bytes_out: row.get(7)?,
            status_2xx: row.get(8)?,
            status_3xx: row.get(9)?,
            status_4xx: row.get(10)?,
            status_5xx: row.get(11)?,
            error_count: row.get(12)?,
        })
    })?;

    rows.collect()
}

pub fn get_cache_metrics_aggregated(
    conn: &Connection,
    from: Option<&str>,
    to: Option<&str>,
    route: Option<&str>,
    interval_secs: i64,
) -> rusqlite::Result<Vec<CacheMetricBucket>> {
    let bucket_expr = format!(
        "strftime('%Y-%m-%dT%H:%M:%SZ', \
         (CAST(strftime('%s', REPLACE(timestamp, ' UTC', '')) AS INTEGER) / {0}) * {0}, \
         'unixepoch')",
        interval_secs
    );

    let mut sql = format!(
        "SELECT {bucket} AS bucket,
                COUNT(*) AS total,
                SUM(CASE WHEN result = 'hit' THEN 1 ELSE 0 END) AS hits,
                SUM(CASE WHEN result = 'miss' THEN 1 ELSE 0 END) AS misses,
                SUM(CASE WHEN result = 'bypass' THEN 1 ELSE 0 END) AS bypasses,
                CAST(SUM(CASE WHEN result = 'hit' THEN 1 ELSE 0 END) AS REAL) / COUNT(*) AS hit_rate
         FROM cache_metrics",
        bucket = bucket_expr
    );

    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = from {
        conditions.push(format!("timestamp >= ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = to {
        conditions.push(format!("timestamp <= ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = route {
        conditions.push(format!("route = ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" GROUP BY bucket ORDER BY bucket ASC");

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(CacheMetricBucket {
            bucket: row.get(0)?,
            total: row.get(1)?,
            hits: row.get(2)?,
            misses: row.get(3)?,
            bypasses: row.get(4)?,
            hit_rate: row.get(5)?,
        })
    })?;

    rows.collect()
}

pub fn get_request_metrics_summary(
    conn: &Connection,
    from: Option<&str>,
    to: Option<&str>,
    group_by: &str,
) -> rusqlite::Result<Vec<RequestMetricSummary>> {
    let group_col = match group_by {
        "method" => "method",
        "status" => "CAST(status_code AS TEXT)",
        "upstream" => "COALESCE(upstream_addr, 'none')",
        // default to route
        _ => "route",
    };

    let mut sql = format!(
        "SELECT {col} AS key,
                COUNT(*) AS count,
                AVG(duration_total_ms) AS avg_duration_ms,
                MIN(duration_total_ms) AS min_duration_ms,
                MAX(duration_total_ms) AS max_duration_ms,
                AVG(duration_upstream_ms) AS avg_upstream_ms,
                SUM(bytes_in) AS bytes_in,
                SUM(bytes_out) AS bytes_out,
                SUM(CASE WHEN early_exit IS NOT NULL THEN 1 ELSE 0 END) AS error_count,
                SUM(CASE WHEN status_code >= 200 AND status_code < 300 THEN 1 ELSE 0 END) AS status_2xx,
                SUM(CASE WHEN status_code >= 300 AND status_code < 400 THEN 1 ELSE 0 END) AS status_3xx,
                SUM(CASE WHEN status_code >= 400 AND status_code < 500 THEN 1 ELSE 0 END) AS status_4xx,
                SUM(CASE WHEN status_code >= 500 THEN 1 ELSE 0 END) AS status_5xx
         FROM request_metrics",
        col = group_col
    );

    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = from {
        conditions.push(format!("timestamp >= ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = to {
        conditions.push(format!("timestamp <= ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(&format!(
        " GROUP BY {col} ORDER BY count DESC",
        col = group_col
    ));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(RequestMetricSummary {
            key: row.get(0)?,
            count: row.get(1)?,
            avg_duration_ms: row.get(2)?,
            min_duration_ms: row.get(3)?,
            max_duration_ms: row.get(4)?,
            avg_upstream_ms: row.get(5)?,
            bytes_in: row.get(6)?,
            bytes_out: row.get(7)?,
            error_count: row.get(8)?,
            status_2xx: row.get(9)?,
            status_3xx: row.get(10)?,
            status_4xx: row.get(11)?,
            status_5xx: row.get(12)?,
        })
    })?;

    rows.collect()
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
