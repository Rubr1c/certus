use std::{sync::Arc, time::Duration};

use parking_lot::Mutex;
use tokio::{sync::mpsc, time};

use crate::{db, schema::types::ReqResSchemaDTO};

#[inline]
pub async fn run(
    conn: Arc<Mutex<rusqlite::Connection>>,
    mut schema_rx: mpsc::Receiver<ReqResSchemaDTO>,
) {
    tokio::spawn(async move {
        let mut batch: Vec<ReqResSchemaDTO> = Vec::with_capacity(100);

        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                Some(schema) = schema_rx.recv() => {
                    batch.push(schema);

                    if batch.len() >= 100 {
                        db::flush::flush_req_res_schema_batch(&conn, &mut batch);
                    }
                },
                _ = interval.tick() => {
                    if !batch.is_empty() {
                        db::flush::flush_req_res_schema_batch(&conn, &mut batch);
                    }
                }
            }
        }
    });
}
