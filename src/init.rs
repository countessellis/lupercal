use crate::client::*;
use crate::convert::*;
use crate::config::*;
use crate::mode::*;
use crate::request::*;
use crate::splash::*;
use crate::server::*;

///////////// Run

pub fn run() {
  log::info!("{}",splash());
  log::info!("{}",version());
  let mode: Mode = Mode::mode();
  let config: Config = Config::default_config(&mode);
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
      convert.convert();
    },
    _ => {
      log::debug!("Running in Client mode.");
      match Client::new(&config) {
        Some(client) => {
          match Request::from_args(&config) {
            Some(request) => {
              let mut request: Option<Request> = client.request(&request);
              while let Some(next) = request {
                request = client.request(&next);
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
