use std::collections::HashMap;

use tracing::field::Visit;

/// Struct for tracing visitor to save tracing fields in a hashmap
#[derive(Default)]
pub struct LogVisitor {
    pub fields: HashMap<String, String>,
}

impl Visit for LogVisitor {
    fn record_debug(
        &mut self,
        field: &tracing::field::Field,
        value: &dyn core::fmt::Debug,
    ) {
        self.fields.insert(field.name().to_string(), format!("{:?}", value));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.fields.insert(field.name().to_string(), format!("{:.2}", value));
    }

    //could be imporved?
    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.fields.insert(field.name().to_string(), format!("{}", value));
    }
}
