use crate::metrics::{CacheMetric, CacheResult, MetricEvent, RequestMetric};

pub fn save_req(
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

pub fn save_reqs(
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

pub fn save_cache(
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

pub fn save(
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

    save_reqs(conn, req_metrics)?;
    save_cache(conn, cache_metrics)?;

    Ok(())
}
