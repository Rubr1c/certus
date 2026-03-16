use crate::{
    db::models::{
        cache_metric::{CacheMetricBucket, CacheMetricRow},
        request_metric::{
            RequestMetricBucket, RequestMetricRow, RequestMetricSummary,
        },
    },
    metrics::types::{CacheMetric, CacheResult, MetricEvent, RequestMetric},
};

pub fn save_req_metric(
    conn: &rusqlite::Connection,
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
    conn: &mut rusqlite::Connection,
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
    conn: &mut rusqlite::Connection,
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
    conn: &mut rusqlite::Connection,
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

pub fn get_request_metrics(
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
    conn: &rusqlite::Connection,
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

pub fn get_request_metrics_aggregated(
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
    conn: &rusqlite::Connection,
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
    conn: &rusqlite::Connection,
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
