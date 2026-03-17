use tokio::sync::mpsc;
use tracing_subscriber::{
    self, EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::logging::{LogEntryDTO, layer::LogChannelLayer};

#[inline]
pub fn run(log_tx: mpsc::Sender<LogEntryDTO>) {
    // maybe make custom writer for this to send to a channel too?
    // not sure that will make a different or not just a thought
    let console_layer = tracing_subscriber::fmt::layer().with_filter(
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info")),
    );

    let db_layer =
        LogChannelLayer { tx: log_tx }.with_filter(EnvFilter::new("debug"));

    let _ = tracing_subscriber::registry()
        .with(console_layer)
        .with(db_layer)
        .try_init();
}
