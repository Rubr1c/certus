use std::collections::HashMap;

use serde_json::Value;

pub fn extract_body_schema(body: &[u8]) -> Option<String> {
    let val: Value = serde_json::from_slice(body).ok()?;
    let obj = val.as_object()?;
    let schema: HashMap<&str, &str> = obj
        .iter()
        .map(|(k, v)| {
            (
                k.as_str(),
                match v {
                    Value::String(_) => "string",
                    Value::Number(_) => "number",
                    Value::Bool(_) => "boolean",
                    Value::Array(_) => "array",
                    Value::Object(_) => "object",
                    Value::Null => "null",
                },
            )
        })
        .collect();
    serde_json::to_string(&schema).ok()
}
