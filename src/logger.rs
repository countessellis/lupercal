use log::{LevelFilter,Level};
use env_logger::TimestampPrecision;
use std::str::FromStr;
use time::macros::format_description;
use tracing_subscriber::{fmt,fmt::time::UtcTime,layer::SubscriberExt,util::SubscriberInitExt,Layer,Registry};

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
  let logformat = fmt::layer().with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET);
  let timeformat = {
    let layer = fmt::layer();
    match DEFAULT_LOG_FORMAT_TIMESTAMP {
      Some(format) => match format {
        TimestampPrecision::Millis => {
          let timer = UtcTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"));
          fmt::layer().with_timer(timer).boxed()
        },
        TimestampPrecision::Micros => {
          let timer = UtcTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:6]"));
          fmt::layer().with_timer(timer).boxed()
        },
        TimestampPrecision::Nanos => {
          let timer = UtcTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:9]"));
          fmt::layer().with_timer(timer).boxed()
        },
        _ => layer.boxed(),
      },
      None => layer.without_time().boxed(),
    }
  };
  let tui_layer = tui_logger::TuiTracingSubscriberLayer;
  let registry = Registry::default().with(tui_layer).with(loglevel).with(timeformat).with(logformat);
  #[cfg(debug_assertions)]
  {
    match DEFAULT_LOG_TARGET {
      env_logger::fmt::Target::Stdout => {
        let stdout_layer = fmt::layer().with_writer(std::io::stdout);
        registry.with(stdout_layer).init();
      },
      __                               => {
        let stderr_layer = fmt::layer().with_writer(std::io::stderr);
        registry.with(stderr_layer).init();
      },
    }
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
            let file_layer = fmt::layer().with_writer(non_blocking);
            registry.with(file_layer).init();
          },
        }
    } else {
      let file_appender = tracing_appender::rolling::daily("./logs", "app.log");
      let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
      _file_guard = Some(guard);
      let file_layer = fmt::layer().with_writer(non_blocking);
      registry.with(file_layer).init();
    }
  }
}
