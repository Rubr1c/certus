
use rusqlite::Connection;

use crate::db::models::LogEntry;

pub fn connect_db() -> Result<Connection, rusqlite::Error> {
    //TODO: Change path and name
    Connection::open("dev.db")
}

pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    //TODO: Change cols
    let querys = vec![
        "CREATE TABLE IF NOT EXISTS logs(id INTEGER PRIMARY KEY AUTOINCREMENT, entry TEXT)",
    ];

    for query in querys {
        conn.execute(query, ())?;
    }
    Ok(())
}

pub fn save_log(conn: &Connection, entry: String) -> rusqlite::Result<()> {
    conn.execute("INSERT INTO logs (entry) VALUES (?1)", [entry])?;

    Ok(())
}

pub fn get_logs(conn: &Connection) -> rusqlite::Result<Vec<LogEntry>> {
    let mut stmt = conn.prepare("SELECT id, entry FROM logs")?;
    let logs = stmt.query_map([], |row| {
        Ok(LogEntry { id: row.get(0)?, entry: row.get(1)? })
    })?;
    let mut log_vec: Vec<LogEntry> = Vec::new();
    for log in logs {
        log_vec.push(log?);
    }
    Ok(log_vec)
}
