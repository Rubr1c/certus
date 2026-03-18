use std::time::Duration;

#[inline(always)]
pub fn connect_db() -> rusqlite::Result<rusqlite::Connection> {
    //TODO: Change path and name
    let conn = rusqlite::Connection::open("dev.db")?;

    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.busy_timeout(Duration::from_millis(5000))?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;

    Ok(conn)
}
