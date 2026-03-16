use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Shape of log table in database
#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: i64,
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields: String,
}

impl LogEntry {
    /// Returns fields converted to hashmap
    pub fn get_fields(&self) -> HashMap<String, String> {
        serde_json::from_str(&self.fields).unwrap_or_default()
    }
}
