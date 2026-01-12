use std::io::{Write,stdout,stdin};
use std::fs;
use url::Url;
use std::path::Path;
use std::env;

use crate::defaults::*;

pub(crate) fn prompt(prompt: String, default: String) -> String {
  print!("{} ",prompt);
  stdout().flush().expect("Oups");
  let mut response = String::new();
  let _ = stdin().read_line(&mut response);
  if response.trim().is_empty() { response = default };
  response.trim_end().to_string()
}

pub(crate) fn build_path(path: &String, prefix: &String) -> String {
  let prefix: String = prefix.strip_suffix('/').unwrap_or(prefix).to_string();
  let mut parts: Vec<&str> = path.split("/").collect();
  if !prefix.is_empty() {
    parts.reverse();
    parts.push(prefix.as_str());
    parts.reverse();
  }
  let directory: String = parts[..parts.len()-1].join("/");
  let file: String = parts[parts.len()-1].to_string();
  if directory.is_empty() {
    file.clone()
  } else {
    match fs::exists(&directory) {
      Ok(true) => directory+"/"+&file,
      Ok(false) => {
        match fs::create_dir_all(&directory) {
          Ok(()) => directory+"/"+&file,
          Err(_) => path.clone(),
        }
      },
      Err(_) => path.clone(),
    }
  }
}

pub(crate) fn build_abs_url(source: &Url, link: &String) -> String {
  let scheme: &str = if link.contains("://") {
    let parts: Vec<&str> = link.split(":").collect();
    parts[0]
  } else { source.scheme() };
  let host: &str = if link.contains("//") {
    let parts: Vec<&str> = link.split("/").collect();
    parts[2]
  } else { source.host_str().unwrap() };
  let path: &str = if link.contains("//") {
    let parts: Vec<&str> = link.split("/").collect();
    &parts[3..].join("/")
  } else {
    link
  };
  format!("{}://{}/{}",scheme,host,path)
}

pub(crate) fn is_image(file: &String) -> Option<String> {
  let path = Path::new(file);
  match path.extension() {
    Some(extension) => {
      match extension.to_str() {
        Some(extension) => {
          match IMAGE_EXTENSIONS.contains(&extension.to_string().as_str()) {
            true  => Some(file.clone()),
            false => None,
          }
        },
        None => None,
      }
    },
    None => None,
  }
}

pub(crate) fn bin_name() -> String {
  match env::current_exe() {
    Ok(path) => {
       let path = path.as_path();
       match path.file_name() {
         Some(file_name) => match file_name.to_str() {
           Some(file_name) => return file_name.to_string(),
           None => {},
         },
         None => {},
       }
    },
    Err(_) => {},
  }
  BUILD_NAME.to_string()
}

