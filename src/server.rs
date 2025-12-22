use std::fmt;
use std::net::TcpListener;
use std::{time,thread,io};
use openssl::ssl::{SslAcceptor,SslMethod};
use openssl::pkey::PKey;
use std::io::ErrorKind;
use std::io::Write;
use std::fs;
use url::Url;
use std::path::Path;

use crate::defaults::*;
use crate::response::*;
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
                      let response: Response = match stream.ssl_read(&mut buffer) {
                        Ok(len) => {
                          match str::from_utf8(&buffer) {
                            Ok(request) => {
                              log::debug!("Request: {}",request.trim_end());
                              match Url::parse(request.trim_end()) {
                                Ok(url) => {
                                  let mut file: String = format!("{}{}",DEFAULT_CONTENT_DIR,url.path());
                                  if file.ends_with("/") { file.truncate(file.len()-1); }
                                  let path = Path::new(&file);
                                  if path.is_dir() { file = format!("{}/index.gmi",file) }
                                  match fs::exists(&file) {
                                    Ok(true) => {
                                      match fs::read_to_string(&file) {
                                        Ok(content) => {
                                          log::info!("Returning contents of {}.",file);
                                          Response::new(&ResponseCode::Success,&String::from("text/gemini"),&content)
                                        },
                                        Err(err) => {
                                          log::error!("Failed to read file {}: {}",file,err);
                                          Response::new(&ResponseCode::Fail,&String::from("Server Error"),&String::new())
                                        },
                                      }
                                    },
                                    Ok(false) => {
                                      log::error!("File {} does not exist.",file);
                                      Response::new(&ResponseCode::FailNotFound,&format!("{} not found",url.path()),&String::new())
                                    },
                                    Err(err) => {
                                      log::error!("Error testing if {} exists: {}",file,err);
                                      Response::new(&ResponseCode::Fail,&String::from("Server Failure"),&String::new())
                                    }
                                  }
                                },
                                Err(err) => {
                                  log::error!("Failed to parse uri from request: {}",err);
                                  Response::new(&ResponseCode::FailBadReq,&String::from("Invalid URI"),&String::new())
                                },
                              }
                            },
                            Err(err) => {
                              log::error!("Failed to parse request: {}",err);
                              Response::new(&ResponseCode::FailBadReq,&String::from("Bad Request"),&String::new())
                            },
                          }
                        },
                        Err(err) => {
                          log::error!("Failed to read request: {}",err);
                          Response::new(&ResponseCode::FailPerm,&String::from("Failed to read request"),&String::new())
                        },
                      };
                      log::debug!("Response: {}",response);
                      stream.write_all(format!("{}",response).as_bytes());
                      stream.shutdown();
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
