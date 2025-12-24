use std::net::TcpListener;
use std::thread;
use openssl::ssl::{SslAcceptor,SslMethod};
use openssl::pkey::PKey;
use std::io::ErrorKind;
use std::io::Write;
use std::fs;
use url::Url;
use std::path::Path;
use std::env::Args;
use std::env::args;

use crate::config::*;
use crate::defaults::*;
use crate::response::*;
use crate::store::*;

///////////// Mode

pub(crate) struct Client {
  pub(crate) config: Config,
  pub(crate) keys: Store,
}

impl Clone for Client {
  fn clone(&self) -> Self {
    Client { config: self.config.clone(), keys: self.keys.clone() }
  }
}

impl Client {
  pub(crate) fn new(config: &Config) -> Option<Client> {
    // Create store:
    match Store::new(&config) {
      Ok(store) => {
        log::info!("Key store created successfully.");
        log::info!("Starting listener...");
        Some(Client { config: config.clone(), keys: store.clone() })
      },
      Err(err)    => {
        log::error!("Failed to create store: {}",err);
        None
      },
    }
  }

  pub(crate) fn request(&self) {
    let mut args: Args = args();
    while let Some(arg) = args.next() {
      if arg.starts_with("gemini://") {
        match Url::parse(&arg) {
          Ok(url) => {
            log::info!("Making request for: {}",arg);
          },
          Err(err) => {
            log::error!("Failed to parse {} as an url: {}",arg,err);
          },
        }
      }
    }
  }
}
