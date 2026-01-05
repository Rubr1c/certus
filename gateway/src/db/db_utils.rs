use rusqlite::Connection;

use crate::db::models::LogEntry;

pub fn connect_db() -> Result<Connection, rusqlite::Error> {
    //TODO: Change path and name
    Connection::open("dev.db")
}


pub fn create_tables(conn: &Connection) {
    todo!()
}

pub fn save_log(conn: &Connection, entry: String) {
    todo!()
}