use crate::db::models::request_metric::{
    RequestMetricBucket, RequestMetricRow,
};
use rusqlite::types::ToSql;

pub fn get(
    conn: &rusqlite::Connection,
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
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();

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

    let param_refs: Vec<&dyn ToSql> =
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

pub fn agg(
    conn: &rusqlite::Connection,
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
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();

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

    let param_refs: Vec<&dyn ToSql> =
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
