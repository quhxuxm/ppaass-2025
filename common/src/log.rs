use crate::Error;
use crate::config::WithLogConfig;
use std::str::FromStr;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::time::ChronoUtc;
pub fn init_log<C: WithLogConfig>(config: &C) -> Result<WorkerGuard, Error> {
    let (trace_file_appender, trace_appender_guard) = tracing_appender::non_blocking(
        tracing_appender::rolling::daily(config.log_directory(), config.log_name_prefix()),
    );
    tracing_subscriber::fmt()
        .with_max_level(Level::from_str(config.max_log_level())?)
        .with_writer(trace_file_appender)
        .with_line_number(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_timer(ChronoUtc::rfc_3339())
        .with_ansi(false)
        .init();
    Ok(trace_appender_guard)
}
