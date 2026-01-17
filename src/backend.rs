use url::Url;

use crate::client::*;
use crate::config::*;

///////////// Backend

pub(crate) enum Backend {
  File(String),
  Http(String),
  Gemini(Client,String),
  Proxy(Client),
  Invalid,
}

impl Backend {
  pub(crate) fn from_source(config: &Config, source: &String) -> Backend {
    match Url::parse(&source) {
      Ok(url) => {
        match url.scheme() {
          "file" => Backend::File(source.clone()),
          "http"   | "https" => Backend::Http(source.clone()),
          "gemini" | "gmi"   => match Client::new(&config) {
            Some(client) => Backend::Gemini(client,source.clone()),
            None         => Backend::Invalid,
          },
          "proxy"            => match Client::new(&config) {
            Some(client) => Backend::Proxy(client),
            None         => Backend::Invalid,
          },
          _                  => Backend::Invalid,
        }
      },
      Err(_) => Backend::File(source.clone()),
    }
  }
}
