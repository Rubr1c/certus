use tokio::sync::mpsc;
use tracing::Subscriber;
use tracing_subscriber::Layer;

use super::{LogEntryDTO, visitor::LogVisitor};

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
        let target = event.metadata().target().to_string();

        let mut visitor = LogVisitor::default();
        event.record(&mut visitor);

        let message = visitor.fields.remove("message").unwrap_or_default();

        let entry = LogEntryDTO {
            timestamp,
            level,
            target,
            message,
            fields: visitor.fields,
        };

        let _ = self.tx.try_send(entry);
    }
}
