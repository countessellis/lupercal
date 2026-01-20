use is_terminal::IsTerminal;
use std::io;

use crate::client::*;
use crate::convert::*;
use crate::config::*;
use crate::mode::*;
use crate::request::*;
use crate::splash::*;
use crate::server::*;

use crate::logger;

///////////// Run

pub fn run() {
  logger::init();
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
      match Server::new(&config) {
        Some(server) => {
          server.listen();
        },
        None => {},
      }
    },
    Mode::Convert => {
      log::debug!("Running in Convert mode.");
      let convert: Convert = Convert::new(&config);
      match convert.run() {
        Ok(())   => log::info!("Finished converting {} into {}.",config.convert_in,config.convert_out),
        Err(err) => log::error!("Conversion of {} into {} failed: {}",config.convert_in,config.convert_out,err),
      }
    },
    _ => {
      log::debug!("Running in Client mode.");
      match Client::new(&config) {
        Some(client) => {
          match Request::from_args(&config) {
            Some(request) => {
              match client.request(&request) {
                Some(response) => {
                  client.tui(&response);
                },
                None => {},
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
