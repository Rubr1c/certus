use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ai::{
        docs::{preprocess, prompt, types::GeneratedApiDocs},
        error::AiError,
        providers::DocsProvider,
    },
    db,
};

const SCHEMA_FETCH_CAP: u32 = 800;

#[inline(always)]
pub async fn generate(
    conn: Arc<Mutex<rusqlite::Connection>>,
) -> Result<GeneratedApiDocs, AiError> {
    let read_conn = Arc::clone(&conn);
    let result = tokio::task::spawn_blocking(move || {
        let conn_guard = read_conn.lock();
        db::repository::schema::get_recent(&conn_guard, SCHEMA_FETCH_CAP)
    })
    .await
    .map_err(|e| AiError::TaskJoin(e.to_string()))?;

    let rows = result.map_err(|e| AiError::DataStore(e.to_string()))?;

    let prepared = preprocess::dedupe_keep_newest(rows);
    if prepared.is_empty() {
        let empty_docs = GeneratedApiDocs {
            title: "API documentation".to_string(),
            introduction: "No request/response schemas are stored yet. Run traffic through the gateway with schema capture enabled; then try generating again.".to_string(),
            highlights: vec![],
            endpoints: vec![],
        };

        save_generated_docs(conn, &empty_docs).await?;

        return Ok(empty_docs);
    }

    let prompt = prompt::build(&prepared)?;
    let provider = DocsProvider::default_from_env()?;
    let docs = provider.generate(&prompt).await?;
    save_generated_docs(conn, &docs).await?;

    Ok(docs)
}

#[inline(always)]
pub async fn latest(
    conn: Arc<Mutex<rusqlite::Connection>>,
) -> Result<Option<GeneratedApiDocs>, AiError> {
    let result = tokio::task::spawn_blocking(move || {
        let conn_guard = conn.lock();
        db::repository::docs::get_latest(&conn_guard)
    })
    .await
    .map_err(|e| AiError::TaskJoin(e.to_string()))?;

    result.map_err(|e| AiError::DataStore(e.to_string()))
}

async fn save_generated_docs(
    conn: Arc<Mutex<rusqlite::Connection>>,
    docs: &GeneratedApiDocs,
) -> Result<(), AiError> {
    let docs = docs.clone();

    let result = tokio::task::spawn_blocking(move || {
        let conn_guard = conn.lock();
        db::repository::docs::save(&conn_guard, &docs)
    })
    .await
    .map_err(|e| AiError::TaskJoin(e.to_string()))?;

    result.map_err(|e| AiError::DataStore(e.to_string()))
}
