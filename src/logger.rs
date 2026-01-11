use log::{LevelFilter,Level};
use std::str::FromStr;
use std::io::Write;
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Registry};
use tracing_subscriber::Layer;

use crate::defaults::*;

///////////// Logger

pub fn init() {
  let loglevel: LevelFilter = match std::env::var("RUST_LOG") {
    Ok(level) => match Level::from_str(level.as_str()) {
      Ok(level) => level.to_level_filter(),
      Err(_)    => DEFAULT_LOG_LEVEL,
    },
    Err(_)    => DEFAULT_LOG_LEVEL,
  };
  let loglevel = tracing_subscriber::filter::LevelFilter::from_str(&loglevel.to_string()).unwrap_or(tracing_subscriber::filter::LevelFilter::INFO);
  let tui_layer = tui_logger::TuiTracingSubscriberLayer;
  let registry = Registry::default().with(tui_layer);
  #[cfg(debug_assertions)]
  {
    let stderr_layer = fmt::layer().with_writer(std::io::stderr).with_filter(loglevel);
    registry.with(stderr_layer).init();
  }
  #[cfg(not(debug_assertions))]
  {
    if std::env::var("JOURNAL_STREAM").is_ok() {
        match tracing_journald::layer() {
          Ok(journal_layer) => {
            registry.with(journal_layer).init();
          },
          Err(_) => {
            let file_appender = tracing_appender::rolling::daily("./logs", "app.log");
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            let _file_guard = Some(guard);
            let file_layer = fmt::layer().with_writer(non_blocking).with_filter(loglevel);
            registry.with(file_layer).init();
          },
        }
    } else {
      let file_appender = tracing_appender::rolling::daily("./logs", "app.log");
      let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
      _file_guard = Some(guard);
      let file_layer = fmt::layer().with_writer(non_blocking).with_filter(loglevel);
      registry.with(file_layer).init();
    }
  }
}
