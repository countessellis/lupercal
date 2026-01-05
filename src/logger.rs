use log::{LevelFilter,Level};
use std::str::FromStr;
use std::io::Write;

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
  let drain = tui_logger::Drain::new();
  env_logger::Builder::from_default_env()
    .filter_level(loglevel)
    .format(move |buf, record| {
      let mut metadata: Vec<String> = Vec::new();
      match DEFAULT_LOG_FORMAT_TIMESTAMP {
        Some(format) => metadata.push(buf.timestamp_millis().to_string()),
        None => {},
      }
      match DEFAULT_LOG_FORMAT_LEVEL {
        true => {
          let level_style = buf.default_level_style(record.level());
          let level = record.level();
          let reset = level_style.render_reset();
          metadata.push(format!("{level_style}{level:<5}{reset}"));
        },
        false => {},
      }
      match DEFAULT_LOG_FORMAT_TARGET {
        true => metadata.push(record.target().to_string()),
        false => {},
      }
      if metadata.len() > 0 {
        //writeln!(buf,"[{}] {}",metadata.join(" "),record.args())?;
      } else {
        writeln!(buf,"{}",record.args())?;
      }
      Ok(drain.log(record))
    })
    .target(DEFAULT_LOG_TARGET)
    .init();
}

