use crate::{
    db::models::cache_metric::{CacheMetricBucket, CacheMetricRow},
    metrics::{CacheMetric, CacheResult},
};
use rusqlite::types::ToSql;

pub fn save(
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

pub fn get(
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
        Ok(CacheMetricRow {
            timestamp: row.get(0)?,
            route: row.get(1)?,
            result: row.get(2)?,
        })
    })?;

    rows.collect()
}

pub fn agg(
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

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" GROUP BY bucket ORDER BY bucket ASC");

    let param_refs: Vec<&dyn ToSql> =
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
