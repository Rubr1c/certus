use std::sync::Arc;

use axum::http::StatusCode;
use parking_lot::Mutex;

pub async fn run<T, F>(
    conn: Arc<Mutex<rusqlite::Connection>>,
    task: F,
    query_error: &'static str,
    panic_error: &'static str,
) -> Result<T, StatusCode>
where
    T: Send + 'static,
    F: FnOnce(&rusqlite::Connection) -> rusqlite::Result<T> + Send + 'static,
{
    let result = tokio::task::spawn_blocking(move || {
        let conn_guard = conn.lock();
        task(&conn_guard)
    })
    .await;

    match result {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(err)) => {
            tracing::error!(err = ?err, context = query_error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(err) => {
            tracing::error!(err = ?err, context = panic_error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
