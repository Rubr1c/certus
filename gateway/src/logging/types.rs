use std::collections::HashMap;

use serde::Serialize;

/// Log entry data transfer object; all fields required
/// to add a new entry to the database
#[derive(Clone, Serialize)]
pub struct LogEntryDTO {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields: HashMap<String, String>,
}
