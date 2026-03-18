use crate::db::models::request_metric::RequestMetricSummary;
use rusqlite::types::ToSql;

#[inline(always)]
pub fn get(
    conn: &rusqlite::Connection,
    from: Option<&str>,
    to: Option<&str>,
    group_by: &str,
) -> rusqlite::Result<Vec<RequestMetricSummary>> {
    let group_col = match group_by {
        "method" => "method",
        "status" => "CAST(status_code AS TEXT)",
        "upstream" => "COALESCE(upstream_addr, 'none')",
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
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();

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

    let param_refs: Vec<&dyn ToSql> =
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
