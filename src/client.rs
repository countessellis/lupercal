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
use std::net::TcpStream;
use openssl::ssl::{SslConnector,SslVerifyMode};
use std::io::{BufReader,BufRead};

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
            match url.host_str() {
              Some(host) => {
                match TcpStream::connect(format!("{}:{}",host,DEFAULT_LISTEN_PORT)) {
                  Ok(connection) => {
                    match SslConnector::builder(SslMethod::tls()) {
                      Ok(mut builder) => {
                        builder.set_verify(SslVerifyMode::NONE);
                        let connector: SslConnector = builder.build();
                        match connector.connect(host, &connection) {
                          Ok(mut tunnel) => {
                            match tunnel.ssl_write(format!("{}\r\n",arg).as_bytes()) {
                              Ok(_) => {
                                let mut buffer: [u8;1024] = [0;1024];
                                match tunnel.ssl_read(&mut buffer) {
                                  Ok(len) => {
                                    match Response::from_bytes(&buffer.to_vec()) {
                                      Ok(response) => {
                                        log::info!("Response: {}",response);
                                      },
                                      Err(err) => {
                                        log::error!("Failed to parse response from {}: {}",arg,err);  
                                      },
                                    }
                                  },
                                  Err(err) => {
                                    log::error!("Failed to read response from {}: {}",arg,err);
                                  },
                                }
                              },
                              Err(err) => {
                                log::error!("Failed to send request to {}: {}",arg,err);
                              },
                            }
                          },
                          Err(err)   => {
                            log::error!("Failed to establish SSL connection to {}:{}: {}",host,DEFAULT_LISTEN_PORT,err);
                          }
                        }
                      },
                      Err(err) => {
                        log::error!("Failed to create SSL connector for {}:{}: {}",host,DEFAULT_LISTEN_PORT,err);
                      },
                    }
                  },
                  Err(err)   => {
                    log::error!("Failed to connect to {}:{}: {}",host,DEFAULT_LISTEN_PORT,err);
                  },
                };
              },
              None => {
                log::error!("Unable to determine host.");
              },
            }
          },
          Err(err) => {
            log::error!("Failed to parse {} as an url: {}",arg,err);
          },
        }
      }
    }
  }
}
