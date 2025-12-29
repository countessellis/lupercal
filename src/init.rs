use crate::client::*;
use crate::config::*;
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
      let config: Config = Config::from_file(&DEFAULT_SERVER_CONFIG.to_string());
      match Server::new(&config) {
        Some(server) => {
          server.listen();
        },
        None => {},
      }
    },
    Mode::Client => {
      log::debug!("Running in Client mode.");
      let config: Config = Config::from_file(&DEFAULT_CLIENT_CONFIG.to_string());
      match Client::new(&config) {
        Some(client) => {
          match Client::request_from_args() {
            Some(request) => {
              let mut request: Option<String> = client.request(&request);
              while let Some(link) = request {
                request = client.request(&link);
              }
            },
            None          => log::error!("No request provided."),
          }
        },
        None => {},
      }
    }
  }
}
