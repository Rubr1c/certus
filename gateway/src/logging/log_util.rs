use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{Subscriber, field::Visit};
use tracing_subscriber::Layer;

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

pub struct LogEntryDTO {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, String>,
}

pub struct LogChannelLayer {
    pub tx: mpsc::Sender<LogEntryDTO>,
}

impl<S> Layer<S> for LogChannelLayer
where
    S: Subscriber,
{
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
