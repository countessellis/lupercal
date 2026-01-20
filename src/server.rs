use std::net::TcpListener;
use std::thread;
use openssl::ssl::{SslAcceptor,SslMethod};
use openssl::pkey::PKey;
use std::io::ErrorKind;
use std::io::Write;
use std::fs;
use std::path::Path;
use openssl::ssl::NameType;
use openssl::hash::MessageDigest;
use openssl::ssl::SslVerifyMode;

use crate::backend::*;
use crate::config::*;
use crate::defaults::*;
use crate::request::*;
use crate::response::*;
use crate::store::*;

///////////// Server

pub(crate) struct Server {
  pub(crate) config: Config,
  pub(crate) keys: Store,
  pub(crate) listener: TcpListener,
  pub(crate) backend: Backend,
}

impl Clone for Server {
  fn clone(&self) -> Self {
    let socket: TcpListener = self.listener.try_clone().unwrap();
    Server { config: self.config.clone(), keys: self.keys.clone(), listener: socket, backend: self.backend.clone() }
  }
}

impl Server {
  pub(crate) fn new(config: &Config) -> Option<Server> {
    // Create store:
    match Store::new(&config) {
      Ok(store) => {
        log::info!("Key store created successfully.");
        log::info!("Initializing backend...");
        let backend: Backend = match Backend::new(&config) {
          Backend::None => return None,
          backend => backend,
        };
        log::info!("Starting listener...");
        let socket: TcpListener = match TcpListener::bind(format!("{}:{}",config.listen_addr,DEFAULT_LISTEN_PORT)) {
          Ok(socket) => {
            log::info!("Server {} listening on {}:{}",config.name,config.listen_addr,DEFAULT_LISTEN_PORT);
            socket
          },
          Err(err)   => {
            log::error!("Failed to initialize listener for control: {}",err);
            return None
          },
        };
        Some(Server { config: config.clone(), keys: store.clone(), listener: socket, backend: backend })
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
            let config: Config = self.config.clone();
            let mut keys: Store = self.keys.clone();
            let backend: Backend = self.backend.clone();
            move || {
              match SslAcceptor::mozilla_modern_v5(SslMethod::tls_server()) {
                Ok(mut builder) => {
                  let allow = config.allow.clone();
                  let deny = config.deny.clone();
                  builder.set_servername_callback(move |ssl, _alerts| {
                    let requested_name = ssl.servername(NameType::HOST_NAME);
                    match requested_name {
                      Some(name) => {
                        if allow.contains(&name.to_string()) {
                          Ok(())
                        } else if !deny.contains(&name.to_string()) {
                          Err(openssl::ssl::SniError::ALERT_FATAL)
                        } else {
                          Ok(())
                        }
                      },
                      None => Err(openssl::ssl::SniError::ALERT_FATAL),
                    }
                  });
                  builder.set_private_key(&PKey::from_rsa(keys.keys.keypair.clone()).unwrap()).unwrap();
                  builder.set_verify(SslVerifyMode::PEER);
                  builder.set_verify_callback(SslVerifyMode::PEER, |_, _| true);
                  builder.set_session_id_context(BUILD_NAME.as_bytes()).unwrap();
                  match builder.set_certificate(&keys.keys.cert.clone()) {
                    Ok(()) => {},
                    Err(err) => {
                      log::error!("Failed to set certificate for TLS: {}",err);
                      return;
                    },
                  }
                  let acceptor: SslAcceptor = builder.build();
                  let mut buffer: [u8;1024] = [0;1024];
                  match acceptor.accept(incoming) {
                    Ok(mut stream) => {
                      match stream.ssl().peer_certificate().or_else(|| { stream.ssl().peer_cert_chain() .and_then(|chain| chain.get(0).map(|c| c.to_owned())) }) {
                        Some(cert) => {
                          let fingerprint: String = match cert.digest(MessageDigest::sha1()) {
                            Ok(fingerprint) => hex::encode(fingerprint),
                            Err(err) => {
                              log::error!("Unable to retrieve fingerprint from cert: {}",err);
                              String::new()
                            },
                          };
                          if !fingerprint.is_empty() {
                             keys.verify(&fingerprint,&cert);
                          }
                        },
                        None => {
                          log::info!("No client certificate provided.");
                        },
                      }
                      let response: Response = match stream.ssl_read(&mut buffer) {
                        Ok(len) => {
                          match Request::from_bytes(&config,&buffer) {
                            Some(request) => {
                              log::debug!("Request: {}, Length: {}",request.next,len);
                              backend.get(&request)
                            },
                            None => {
                              log::error!("Request format wrong.");
                              Response::new(&ResponseCode::FailBadReq,&String::from("Bad Request"),&Vec::new(),&None)
                            },
                          }
                        },
                        Err(err) => {
                          log::error!("Failed to read request: {}",err);
                          Response::new(&ResponseCode::FailPerm,&String::from("Failed to read request"),&Vec::new(),&None)
                        },
                      };
                      log::debug!("Response: {}",response);
                      match stream.write_all(&response.into_bytes()) {
                        Ok(_) => {},
                        Err(err) => log::error!("Failed to send response: {}",err),
                      }
                      match stream.shutdown() {
                        Ok(_) => {},
                        Err(err) => log::error!("Failed to close connection: {}",err),
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
