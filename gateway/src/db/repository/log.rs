use crate::{db::models::log::LogEntry, logging::LogEntryDTO};
use rusqlite::types::ToSql;

/// Saves a log to an sqlite database
///
/// # Arguments
///
/// * `conn` - connection to sqlite database
/// * `entry` - data of the log to save
///
/// # Errors
///
/// Returns an error if:
/// * Failed to execute query
pub fn save_one(
    conn: &rusqlite::Connection,
    entry: LogEntryDTO,
) -> rusqlite::Result<()> {
    let fields_json = serde_json::to_string(&entry.fields)
        .unwrap_or_else(|_| "{}".to_string());

    conn.execute(
        "INSERT INTO logs (timestamp, level, target, message, fields) 
                  VALUES (?1, ?2, ?3, ?4, ?5)",
        [
            entry.timestamp,
            entry.level,
            entry.target,
            entry.message,
            fields_json,
        ],
    )?;

    Ok(())
}

/// Saves a vec of logs transactionaly
///
/// # Arguments
///
/// * `conn` - mutable connection to sqlite database
/// * `entries` - vector of logs to save
///
/// # Errors
///
/// Returns an error if:
/// * Failed to start transaction
/// * Failed to prepare query
/// * Failed to execute query
/// * Failed to commit transaction
pub fn save(
    conn: &mut rusqlite::Connection,
    entries: Vec<LogEntryDTO>,
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    {
        let mut query = tx.prepare(
            "INSERT INTO logs (timestamp, level, target, message, fields)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;

        for entry in entries {
            let fields_json = serde_json::to_string(&entry.fields)
                .unwrap_or_else(|_| "{}".to_string());

            query.execute([
                entry.timestamp,
                entry.level,
                entry.target,
                entry.message,
                fields_json,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Returns all logs from an sqlite database
///
/// # Arguments
///
/// * `conn` - connection to sqlite database
///
/// # Errors
///
/// Returns an error if:
/// * Failed to prepare query
/// * Failed to get element
/// * Failed to map query
pub fn all(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<LogEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, level, target, message, fields FROM logs",
    )?;

    let logs = stmt.query_map([], |row| {
        Ok(LogEntry {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            level: row.get(2)?,
            target: row.get(3)?,
            message: row.get(4)?,
            fields: row.get(5)?,
        })
    })?;
    let mut log_vec: Vec<LogEntry> = Vec::new();
    for log in logs {
        log_vec.push(log?);
    }
    Ok(log_vec)
}

pub fn get(
    conn: &rusqlite::Connection,
    from: Option<&str>,
    to: Option<&str>,
    level: Option<&str>,
    target: Option<&str>,
    search: Option<&str>,
    page: u32,
    page_size: u32,
) -> rusqlite::Result<Vec<LogEntry>> {
    let mut sql = String::from(
        "SELECT id, timestamp, level, target, message, fields FROM logs",
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
    if let Some(v) = level {
        conditions.push(format!("level = ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = target {
        conditions.push(format!("target = ?{}", params.len() + 1));
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = search {
        conditions.push(format!("message LIKE ?{}", params.len() + 1));
        params.push(Box::new(format!("%{}%", v)));
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
        Ok(LogEntry {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            level: row.get(2)?,
            target: row.get(3)?,
            message: row.get(4)?,
            fields: row.get(5)?,
        })
    })?;

    rows.collect()
}
