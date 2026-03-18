use crate::config::Config;
use rusqlite::types::Type;

/// Saves gateway config to sqlite database
///
/// # Arguments
///
/// * `conn` - connection to sqlite database
/// * `config` - config struct to save
///
/// # Errors
///
/// Returns an error if:
/// * Failed to execute query
///
/// # Panics
///
/// Panics if:
/// * Failed to parse config to json
#[inline(always)]
pub fn save(
    conn: &rusqlite::Connection,
    config: &Config,
) -> rusqlite::Result<()> {
    let json_str = serde_json::to_string(config)
        .expect("failed to serialize config to JSON");

    conn.execute(
        "INSERT INTO config (id, config_data, updated_at)
             VALUES (1, ?1, CURRENT_TIMESTAMP)
                ON CONFLICT(id) DO UPDATE SET 
                config_data = excluded.config_data,
                updated_at = CURRENT_TIMESTAMP
            ",
        [json_str],
    )?;

    Ok(())
}

/// Gets config from sqlite database
///
/// # Arguments
///
/// * `conn` - connection to sqlite database
///
/// # Errors
///
/// Returns an error if:
/// * Failed to get config row
/// * Failed to parse json to config
#[inline(always)]
pub fn get(conn: &rusqlite::Connection) -> rusqlite::Result<Config> {
    conn.query_row("SELECT config_data FROM config WHERE id = 1", [], |row| {
        let json_str: String = row.get(0)?;
        serde_json::from_str(&json_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                Type::Text,
                Box::new(e),
            )
        })
    })
}
