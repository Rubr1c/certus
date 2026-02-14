use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: i64,
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: String,
}

impl LogEntry {
    pub fn get_fields(&self) -> HashMap<String, String> {
        serde_json::from_str(&self.fields).unwrap_or_default()
    }
}
