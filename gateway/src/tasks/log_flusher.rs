use std::{sync::Arc, time::Duration};

use parking_lot::Mutex;
use tokio::{
    sync::{broadcast, mpsc},
    time,
};

use crate::{db, logging::LogEntryDTO};

#[inline]
pub async fn run(
    conn: Arc<Mutex<rusqlite::Connection>>,
    mut log_rx: mpsc::Receiver<LogEntryDTO>,
    log_broadcast_tx: Option<broadcast::Sender<LogEntryDTO>>,
) {
    tokio::spawn(async move {
        let mut batch: Vec<LogEntryDTO> = Vec::with_capacity(100);

        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                Some(entry) = log_rx.recv() => {
                    batch.push(entry.clone());
                    if let Some(tx) = &log_broadcast_tx {
                        let _ = tx.send(entry);
                    }

                    if batch.len() >= 100 {
                        db::flush::flush_log_batch(&conn, &mut batch);
                    }
                }

                _ = interval.tick() => {
                    if !batch.is_empty() {
                        db::flush::flush_log_batch(&conn, &mut batch);
                    }
                }
            }
        }
    });
}
