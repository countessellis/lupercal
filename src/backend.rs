use url::Url;
use std::path::Path;
use std::fs;

use crate::client::*;
use crate::config::*;
use crate::request::*;
use crate::response::*;

///////////// Backend

#[derive(Clone)]
pub(crate) enum Backend {
  File(String),
  Http(String),
  Gemini(Client,String),
  Proxy(Client),
  None,
}

impl Backend {
  pub(crate) fn new(config: &Config) -> Backend {
    match Url::parse(&config.source) {
      Ok(url) => {
        match url.scheme() {
          "file" => Backend::File(config.source.clone()),
          "http"   | "https" => Backend::Http(config.source.clone()),
          "gemini" | "gmi"   => match Client::new(&config) {
            Some(client) => Backend::Gemini(client,config.source.clone()),
            None         => Backend::None,
          },
          "proxy"            => match Client::new(&config) {
            Some(client) => Backend::Proxy(client),
            None         => Backend::None,
          },
          _                  => Backend::None,
        }
      },
      Err(_) => Backend::File(config.source.clone()),
    }
  }

  pub(crate) fn get(&self, request: &Request) -> Response {
    match self {
      Backend::File(source) => {
        match request.as_url() {
          Some(url) => {
            let mut file: String = format!("{}{}",source,url.path());
            if file.ends_with("/") { file.truncate(file.len()-1); }
            let path = Path::new(&file);
            if path.is_dir() { file = format!("{}/index.gmi",file) }
            match fs::exists(&file) {
              Ok(true) => {
                let path = Path::new(&file);
                let mimetype: String = if let Some(extension) = path.extension() {
                  match extension.to_str() {
                    Some("gmi")     => String::from("text/gemini"),
                    Some("gemini")  => String::from("text/gemini"),
                    Some(extension) => match mime_guess::from_ext(extension).first() {
                      Some(guess)   => guess.essence_str().to_string(),
                      None          => String::from("application/octet-stream"),
                    },
                    None            => String::from("application/octet-stream"),
                  }
                } else { String::from("application/octet-stream") };
                match fs::read(&file) {
                  Ok(content) => {
                    log::info!("Returning contents of {}.",file);
                    return Response::new(&ResponseCode::Success,&String::from(mimetype),&content,&Some(request.clone()))
                  },
                  Err(err) => {
                    log::error!("Failed to read file {}: {}",file,err);
                    return Response::new(&ResponseCode::Fail,&String::from("Server Error"),&Vec::new(),&Some(request.clone()))
                  },
                }
              },
              Ok(false) => {
                log::error!("File {} does not exist.",file);
                Response::new(&ResponseCode::FailNotFound,&format!("{} not found",url.path()),&Vec::new(),&Some(request.clone()))
              },
              Err(err) => {
                log::error!("Error testing if {} exists: {}",file,err);
                Response::new(&ResponseCode::Fail,&String::from("Server Failure"),&Vec::new(),&Some(request.clone()))
              }
            }
          },
          None => {
            log::error!("Failed to parse uri from request.");
            return Response::new(&ResponseCode::FailBadReq,&String::from("Invalid URI"),&Vec::new(),&Some(request.clone()))
          },
        }
      },
      _ => {
        log::error!("Unsupported backend.");
        return Response::new(&ResponseCode::Fail,&String::from("Server Error"),&Vec::new(),&Some(request.clone()))
      },
    }
  }
}
