use log::{LevelFilter,Level};
use std::str::FromStr;

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
  env_logger::Builder::from_default_env()
    .filter_level(loglevel)
    .format_timestamp(DEFAULT_LOG_FORMAT_TIMESTAMP)
    .format_level(DEFAULT_LOG_FORMAT_LEVEL)
    .format_target(DEFAULT_LOG_FORMAT_TARGET)
    .target(DEFAULT_LOG_TARGET)
    .init();
}

