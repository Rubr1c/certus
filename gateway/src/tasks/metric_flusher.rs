use std::{sync::Arc, time::Duration};

use parking_lot::Mutex;
use tokio::{
    sync::{broadcast, mpsc},
    time,
};

use crate::{db, metrics::types::MetricEvent};

#[inline]
pub async fn run(
    conn: Arc<Mutex<rusqlite::Connection>>,
    mut metrics_rx: mpsc::Receiver<MetricEvent>,
    metrics_broadcast_tx: Option<broadcast::Sender<MetricEvent>>,
) {
    tokio::spawn(async move {
        let mut batch: Vec<MetricEvent> = Vec::with_capacity(100);

        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                Some(metric) = metrics_rx.recv() => {
                   if let Some(tx) = &metrics_broadcast_tx {
                       let _ = tx.send(metric.clone());
                   }
                   batch.push(metric);

                   if batch.len() >= 100 {
                        db::flush::flush_metric_batch(&conn, &mut batch);
                   }
                },
               _ = interval.tick() => {
                   if !batch.is_empty() {
                        db::flush::flush_metric_batch(&conn, &mut batch);
                   }
                }
            }
        }
    });
}
