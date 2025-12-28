use std::io;
use tokio::sync::mpsc;

pub struct LogChannelWriter {
    pub sender: mpsc::Sender<String>,
}

impl io::Write for LogChannelWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let log_entry = String::from_utf8_lossy(buf).to_string();

        match self.sender.try_send(log_entry) {
            Ok(_) => {},
            Err(e) => eprintln!("Log Dropped (channel full): {}", e)
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
