use std::fmt;
use std::env::{args,Args};

use crate::config::*;

///////////// Request

#[derive(Debug, Clone)]
pub(crate) struct Request {
  pub(crate) config: Config,
  pub(crate) next: String,
  pub(crate) prev: Box<Option<Request>>,
}

impl fmt::Display for Request {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f,"{}\r\n",self.next)
  }
}

impl Request {
  pub(crate) fn new(config: &Config, next: &String, prev: &Option<Request>) -> Request {
    Request { config: config.clone(), next: next.clone(), prev: Box::new(prev.clone()) }
  }

  pub(crate) fn from_args(config: &Config) -> Option<Request> {
    let mut args: Args = args();
    while let Some(arg) = args.next() {
      if arg.starts_with("gemini://") {
        return Some(Self::new(&config,&arg,&None));
      }
    }
    None
  }

  pub(crate) fn prev(&self) -> Option<Request> {
    *self.prev.clone()
  }

  pub(crate) fn as_bytes(&self) -> Vec<u8> {
    self.to_string().into_bytes()
  }

  pub(crate) fn from_bytes(config: &Config, bytes: &[u8]) -> Option<Request> {
    match str::from_utf8(&bytes) {
      Ok(request) => Some(Request::new(&config.clone(),&request.trim_end().to_string(),&Box::new(None))),
      Err(err) => {
        log::error!("Failed to parse request: {}",err);
        None
      },
    }
  }
}
