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

  let tui_layer = tui_logger::TuiTracingSubscriberLayer;
  tui_logger::init_logger(tui_logger::LevelFilter::Info).unwrap();
  let filter_layer = tracing_subscriber::filter::LevelFilter::from_str(&loglevel.to_string()).unwrap_or(tracing_subscriber::filter::LevelFilter::INFO);

  #[cfg(debug_assertions)]
  {

    let fmt_layer = match DEFAULT_LOG_FORMAT_TIMESTAMP {
      Some(TimestampPrecision::Seconds) => {
        let timer = UtcTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"));
        match DEFAULT_LOG_TARGET {
          env_logger::fmt::Target::Stdout => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(std::io::stdout).boxed(),
          _ => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(std::io::stderr).boxed(),
        }
      },
      Some(TimestampPrecision::Millis) => {
        let timer = UtcTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"));
        match DEFAULT_LOG_TARGET {
          env_logger::fmt::Target::Stdout => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(std::io::stdout).boxed(),
          _ => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(std::io::stderr).boxed(),
        }
      },
      Some(TimestampPrecision::Micros) => {
        let timer = UtcTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:6]"));
        match DEFAULT_LOG_TARGET {
          env_logger::fmt::Target::Stdout => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(std::io::stdout).boxed(),
          _ => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(std::io::stderr).boxed(),
        }
      },
      Some(TimestampPrecision::Nanos) => {
        let timer = UtcTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:9]"));
        match DEFAULT_LOG_TARGET {
          env_logger::fmt::Target::Stdout => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(std::io::stdout).boxed(),
          _ => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(std::io::stderr).boxed(),
        }
      },
      None                            => {
        match DEFAULT_LOG_TARGET {
          env_logger::fmt::Target::Stdout => fmt::layer().without_time().with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(std::io::stdout).boxed(),
          _ => fmt::layer().without_time().with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(std::io::stderr).boxed(),
        }
      },
    };
    let registry = Registry::default().with(tui_layer).with(filter_layer).with(fmt_layer);
    registry.init()
  }

  #[cfg(not(debug_assertions))]
  {
    if std::env::var("JOURNAL_STREAM").is_ok() {
        match tracing_journald::layer() {
          Ok(journal_layer) => {
            let fmt_layer = fmt::layer().with_level(DEFAULT_LOG_FORMAT_LEVEL);
            let registry = Registry::default().with(tui_layer).with(filter_layer).with(fmt_layer);
            registry.with(journal_layer).init();
            return;
          },
          Err(_) => {},
        }
    }
    let file_appender = tracing_appender::rolling::daily("./logs", "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let _file_guard = Some(guard);
    let fmt_layer = match DEFAULT_LOG_FORMAT_TIMESTAMP {
      Some(TimestampPrecision::Seconds) => {
        let timer = UtcTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"));
        match DEFAULT_LOG_TARGET {
          env_logger::fmt::Target::Stdout => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(std::io::stdout).boxed(),
          _ => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(non_blocking).boxed(),
        }
      },
      Some(TimestampPrecision::Millis) => {
        let timer = UtcTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"));
        match DEFAULT_LOG_TARGET {
          env_logger::fmt::Target::Stdout => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(std::io::stdout).boxed(),
          _ => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(non_blocking).boxed(),
        }
      },
      Some(TimestampPrecision::Micros) => {
        let timer = UtcTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:6]"));
        match DEFAULT_LOG_TARGET {
          env_logger::fmt::Target::Stdout => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(std::io::stdout).boxed(),
          _ => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(non_blocking).boxed(),
        }
      },
      Some(TimestampPrecision::Nanos) => {
        let timer = UtcTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:9]"));
        match DEFAULT_LOG_TARGET {
          env_logger::fmt::Target::Stdout => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(std::io::stdout).boxed(),
          _ => fmt::layer().with_timer(timer).with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(non_blocking).boxed(),
        }
      },
      None                            => {
        match DEFAULT_LOG_TARGET {
          env_logger::fmt::Target::Stdout => fmt::layer().without_time().with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(std::io::stdout).boxed(),
          _ => fmt::layer().without_time().with_level(DEFAULT_LOG_FORMAT_LEVEL).with_target(DEFAULT_LOG_FORMAT_TARGET).with_writer(non_blocking).boxed(),
        }
      },
    };
    let registry = Registry::default().with(tui_layer).with(filter_layer).with(fmt_layer);
    registry.init();
  }
}
