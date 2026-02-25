use std::fs::OpenOptions;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn initilize_logger() -> WorkerGuard {
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("ironboyadvance.log")
        .expect("Failed to create log file");

    let (non_blocking, guard) = tracing_appender::non_blocking(log_file);

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .without_time() // remove this
                .with_target(false) // remove this
                .with_level(false) // remove this
                .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))),
        )
        .init();

    guard
}
