/// Runs all queries to create tables and initalize db
///
/// # Arguments
///
/// * `conn` - connection to sqlite database
///
/// # Errors
///
/// Returns an error if:
/// * query is failed to execute
#[inline(always)]
pub fn migrate(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    let querys = vec![
        "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            level TEXT NOT NULL,
            target TEXT NOT NULL,
            message TEXT NOT NULL,
            fields TEXT NOT NULL
        );",
        "CREATE TABLE IF NOT EXISTS config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            config_data TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );",
        "CREATE TABLE IF NOT EXISTS request_metrics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            route TEXT NOT NULL,
            status_code INTEGER NOT NULL,
            duration_total_ms INTEGER NOT NULL,
            duration_upstream_ms INTEGER NOT NULL,
            bytes_in INTEGER NOT NULL,
            bytes_out INTEGER NOT NULL,
            client_ip TEXT NOT NULL,
            method TEXT NOT NULL,
            upstream_addr TEXT,
            early_exit TEXT
        )",
        "CREATE TABLE IF NOT EXISTS cache_metrics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            route TEXT NOT NULL,
            result TEXT NOT NULL
        )",
        "CREATE TABLE IF NOT EXISTS req_res_schemas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            full_path TEXT NOT NULL,
            method TEXT NOT NULL,
            query_params TEXT,
            status_code INTEGER NOT NULL,
            has_auth INTEGER NOT NULL DEFAULT 0,
            req_headers TEXT NOT NULL,
            res_headers TEXT NOT NULL,
            body_schema TEXT
        )",
        "CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs (timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_request_metrics_timestamp ON request_metrics (timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_cache_metrics_timestamp ON cache_metrics (timestamp)",
    ];

    for query in querys {
        conn.execute(query, ())?;
    }

    Ok(())
}
