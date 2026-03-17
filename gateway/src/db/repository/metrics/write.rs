use crate::{db::repository::metrics, metrics::MetricEvent};

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

    metrics::request::save(conn, req_metrics)?;
    metrics::cache::save(conn, cache_metrics)?;

    Ok(())
}
