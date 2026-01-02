use std::io;
use tokio::sync::mpsc;

pub struct LogChannelWriter {
    pub sender: mpsc::Sender<Vec<u8>>,
}

impl io::Write for LogChannelWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let log_data = buf.to_vec();

        match self.sender.try_send(log_data) {
            Ok(_) => {},
            Err(e) => tracing::error!("Log Dropped (channel full): {}", e)
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LogChannelWriter {
    type Writer = LogChannelWriter;

    fn make_writer(&'a self) -> Self::Writer {
        LogChannelWriter {
            sender: self.sender.clone(),
        }
    }
}
