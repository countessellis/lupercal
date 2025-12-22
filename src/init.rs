use std::fs::write;

use crate::defaults::*;
use crate::mode::*;
use crate::splash::*;
use crate::server::*;

///////////// Run

pub fn run() {
  log::info!("{}",splash());
  log::info!("{}",version());
  match Mode::mode() {
    Mode::Server=> {
      log::debug!("Running in Server mode.");
      match Server::new(String::from(SERVER_NAME)) {
        Some(server) => {
          server.listen();
        },
        None => {},
      }
    },
    Mode::Client => {
      log::debug!("Running in Client mode.");
    },
  }
}
