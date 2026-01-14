use std::io;
use is_terminal::IsTerminal;

use crate::client::*;
use crate::convert::*;
use crate::config::*;
use crate::mode::*;
use crate::request::*;
use crate::response::*;
use crate::splash::*;
use crate::server::*;

///////////// Run

pub fn run() {
  if io::stdout().is_terminal() {
    println!("{}",splash());
  } else {
    log::info!("{}",raw_splash());
  }
  log::info!("{}",version());
  let mode: Mode = Mode::mode();
  let config: Config = Config::default_config(&mode);
  log::debug!("Config: {:?}",config);
  match mode {
    Mode::Server=> {
      log::debug!("Running in Server mode.");
      match Server::new(&config) {
        Some(server) => {
          server.listen();
        },
        None => {},
      }
    },
    Mode::Proxy => {
      log::debug!("Running in Proxy mode.");
    },
    Mode::Convert => {
      log::debug!("Running in Convert mode.");
      let convert: Convert = Convert::new(&config);
      convert.run();
    },
    _ => {
      log::debug!("Running in Client mode.");
      match Client::new(&config) {
        Some(client) => {
          match Request::from_args(&config) {
            Some(mut request) => {
              loop {
                match client.request(&request) {
                  Some(response) => {
                    match client.display(&response) {
                      Some(next) => request = next,
                      None => break,
                    }
                  },
                  None => break,
                }
              }
            },
            None          => log::error!("No request provided."),
          }
        },
        None => {},
      }
    },
  }
}
