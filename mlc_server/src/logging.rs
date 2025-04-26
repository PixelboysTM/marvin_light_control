use mlc_data::DynamicResult;
use std::fs::OpenOptions;
use std::io::Write;
use tracing::level_filters::LevelFilter;
use tracing::Level;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::format::{FmtSpan, Writer};
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::{layer, MakeWriter};

pub fn setup_logging() -> DynamicResult<std::sync::mpsc::Receiver<Vec<u8>>> {
    use tracing_subscriber::prelude::*;

    let (w, rx) = FuturesWriter::new();

    let debug = {
        #[cfg(not(debug_assertions))]
        let d = false;

        #[cfg(debug_assertions)]
        let d = true;

        d
    };

    let sub = tracing_subscriber::Registry::default()
        .with(
            layer()
                .compact()
                .with_ansi(false)
                .with_writer(
                    OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open("server.log")?,
                )
                .with_filter(LevelFilter::from_level(Level::WARN)),
        )
        .with(
            layer()
                .with_ansi(false)
                .with_writer(
                    OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open("server-verbose.log")?,
                )
                .with_span_events(FmtSpan::FULL)
                .with_filter(LevelFilter::TRACE),
        )
        .with(
            layer()
                .compact()
                .with_writer(w)
                .with_timer(NiceTime)
                .with_ansi(true)
                // .with_target(true)
                .with_file(debug)
                .with_line_number(debug)
                .with_target(false)
                .with_filter(LevelFilter::INFO),
        );

    tracing::subscriber::set_global_default(sub)?;

    LogTracer::init()?;
    Ok(rx)
}

pub struct NiceTime;

impl FormatTime for NiceTime {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", chrono::Local::now().format("[%H:%M.%S]"))
    }
}

struct FuturesWriter {
    sink: std::sync::mpsc::Sender<Vec<u8>>,
}

impl FuturesWriter {
    fn new() -> (Self, std::sync::mpsc::Receiver<Vec<u8>>) {
        let (tx, rx) = std::sync::mpsc::channel();

        (Self { sink: tx }, rx)
    }
}

impl Write for FuturesWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.sink
            .send(buf.to_vec())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for FuturesWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        Self {
            sink: self.sink.clone(),
        }
    }
}
