use rusqlite::types::Type;

use crate::ai::docs::GeneratedApiDocs;

#[inline(always)]
pub fn save(
    conn: &rusqlite::Connection,
    docs: &GeneratedApiDocs,
) -> rusqlite::Result<()> {
    let json = serde_json::to_string(docs)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    conn.execute(
        "INSERT INTO generated_api_docs (id, docs_json, updated_at)
         VALUES (1, ?1, CURRENT_TIMESTAMP)
         ON CONFLICT(id) DO UPDATE SET
             docs_json = excluded.docs_json,
             updated_at = CURRENT_TIMESTAMP",
        [json],
    )?;

    Ok(())
}

#[inline(always)]
pub fn get_latest(
    conn: &rusqlite::Connection,
) -> rusqlite::Result<Option<GeneratedApiDocs>> {
    let mut stmt =
        conn.prepare("SELECT docs_json FROM generated_api_docs WHERE id = 1")?;

    let row = stmt.query_row([], |row| row.get::<_, String>(0));

    match row {
        Ok(json) => {
            let docs = serde_json::from_str(&json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    Type::Text,
                    Box::new(e),
                )
            })?;
            Ok(Some(docs))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err),
    }
}
