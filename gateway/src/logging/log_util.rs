use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{Subscriber, field::Visit};
use tracing_subscriber::Layer;

/// Struct for tracing visitor to save tracing fields in a hashmap
#[derive(Default)]
struct LogVisitor {
    fields: HashMap<String, String>,
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

/// Log entry data transfer object; all fields required
/// to add a new entry to the database
pub struct LogEntryDTO {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, String>,
}

/// Layer for tracing subscriber to get fields form events
/// and send them to a channel to save to database
pub struct LogChannelLayer {
    pub tx: mpsc::Sender<LogEntryDTO>,
}

impl<S> Layer<S> for LogChannelLayer
where
    S: Subscriber,
{
    /// Takes the event from tracing and extracts the fields from
    /// the event and contructs a [`LogEntryDTO`] to be sent to
    /// the channel to be saved
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let level = event.metadata().level().to_string();

        let mut visitor = LogVisitor::default();
        event.record(&mut visitor);

        let message = visitor.fields.remove("message").unwrap_or_default();

        let entry =
            LogEntryDTO { timestamp, level, message, fields: visitor.fields };

        let _ = self.tx.try_send(entry);
    }
}
