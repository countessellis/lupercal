use std::fmt;
use std::net::TcpListener;
use std::{time,thread,io};
use openssl::ssl::{SslAcceptor,SslMethod};
use openssl::pkey::PKey;
use std::io::ErrorKind;

use crate::defaults::*;
use crate::store::*;

///////////// Mode

pub(crate) struct Server {
  pub(crate) keys: Store,
  pub(crate) listener: TcpListener,
}

impl Clone for Server {
  fn clone(&self) -> Self {
    let socket: TcpListener = self.listener.try_clone().unwrap();
    Server { keys: self.keys.clone(), listener: socket }
  }
}

impl Server {
  pub(crate) fn new(server_name: String) -> Option<Server> {
    // Create store:
    match Store::new(&server_name) {
      Ok(store) => {
        log::info!("Key store created successfully.");
        log::info!("Starting listener...");
        let socket: TcpListener = match TcpListener::bind(DEFAULT_LISTEN_ADDRESS) {
          Ok(socket) => {
            log::info!("Server {} istening on {}",server_name,DEFAULT_LISTEN_ADDRESS);
            socket
          },
          Err(err)   => {
            log::error!("Failed to initialize listener for control: {}",err);
            return None
          },
        };
        Some(Server { keys: store.clone(), listener: socket })
      },
      Err(err)    => {
        log::error!("Failed to create store: {}",err);
        None
      },
    }
  }

  pub(crate) fn listen(&self) {
    self.listener.set_nonblocking(true).unwrap();
    for incoming in self.listener.incoming() {
      match incoming {
        Ok(incoming) => {
          thread::spawn({
            let keys: Store = self.keys.clone();
            move || {
              match SslAcceptor::mozilla_modern_v5(SslMethod::tls_server()) {
                Ok(mut builder) => {
                  builder.set_private_key(&PKey::from_rsa(keys.keys.keypair.clone()).unwrap()).unwrap();
                  builder.set_certificate(&keys.keys.cert.clone().unwrap()).unwrap();
                  let acceptor: SslAcceptor = builder.build();
                  let mut buffer: [u8;1024] = [0;1024];
                  match acceptor.accept(incoming) {
                    Ok(mut stream) => {
                      match stream.ssl_read(&mut buffer) {
                        Ok(len) => {
                          match str::from_utf8(&buffer) {
                            Ok(request) => {
                              log::debug!("Request: {}",request);
                            },
                            Err(err) => log::error!("Failed to parse request: {}",err),
                          }
                        },
                        Err(err) => log::error!("Failed to read request: {}",err),
                      }
                    },
                    Err(err) => log::error!("Failed to establish encryption: {}",err),
                  }
                },
                Err(err) => log::error!("Failed to create encryption acceptor: {}",err),
              }
            }
          });
        }
        Err(err)   => if err.kind() != ErrorKind::WouldBlock { log::error!("Failed to accept connection: {}",err) },
      }
    }
  }
}
