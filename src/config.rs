use std::fs::{read_to_string,write};
use std::fs;
use std::env::{args,Args};
use std::str::FromStr;

use crate::defaults::*;
use crate::mode::*;
use crate::util::*;

#[derive(Debug, Clone)]
pub struct Config {
  // General:
  pub(crate) mode:      Mode,
  // Server related:
  pub(crate) server_name: String,
  pub(crate) content_dir: String,
  pub(crate) server_cache_dir: String,
  pub(crate) server_store_dir: String,
  // Client related:
  pub(crate) client_name: String,
  pub(crate) client_cache_dir: String,
  pub(crate) client_store_dir: String,
}

impl Config {
  pub(crate) fn defaults() -> Config {
    Config {
      // General:
      mode:             Default::default(),
      // Server related:
      server_name:      DEFAULT_SERVER_NAME.to_string(),
      content_dir:      DEFAULT_CONTENT_DIR.to_string(),
      server_cache_dir: DEFAULT_SERVER_CACHE_DIR.to_string(),
      server_store_dir: DEFAULT_SERVER_STORE_DIR.to_string(),
      // Client related:
      client_name:      DEFAULT_CLIENT_NAME.to_string(),
      client_cache_dir: DEFAULT_CLIENT_CACHE_DIR.to_string(),
      client_store_dir: DEFAULT_CLIENT_STORE_DIR.to_string(),
    }
  }
  
  pub fn write(&self,config_file: &String) -> Result<String,String> {
    let config_file: String = build_path(&config_file,&"config".to_string());
    let mut config: Vec<String> = Vec::new();
    // General:
    config.push(format!("mode: {}",self.mode));
    // Server related:
    config.push(format!("server_name: {}",self.server_name));
    config.push(format!("content_dir: {}",self.content_dir));
    config.push(format!("server_cache_dir: {}",self.server_cache_dir));
    config.push(format!("server_store_dir: {}",self.server_store_dir));
    // Client related:
    config.push(format!("client_name: {}",self.client_name));
    config.push(format!("client_cache_dir: {}",self.client_cache_dir));
    config.push(format!("client_store_dir: {}",self.client_store_dir));
    // Build:
    config.push("".to_string());
    match write(&config_file,config.join("\n")) {
      Ok(()) => Ok(format!("Outputted config to {}",config_file)),
      Err(err) => Err(format!("Failed to output config to {}: {}",config_file,err.to_string()))
    }
  }

  pub fn new() -> Config {
    let mut config = Config::defaults();
    println!("\n");
    // General:
    config.mode = match Mode::from_str(prompt(format!("Mode: (server or client, default: {})",config.mode),config.mode.to_string()).as_str()) {
      Ok(mode) => mode,
      Err(_)   => Default::default(),
    };
    // Server related:
    config.server_name = prompt(format!("Server fully qualified domain name: (default: {})",config.server_name),config.server_name);
    config.content_dir = prompt(format!("Content directory: (default: {})",config.content_dir),config.content_dir);
    config.server_cache_dir = prompt(format!("Server cache directory: (default: {})",config.server_cache_dir),config.server_cache_dir);
    config.server_store_dir = prompt(format!("Server store directory: (default: {})",config.server_store_dir),config.server_store_dir);
    // Client related:
    config.client_name = prompt(format!("Client identifying name: (typically username@hostname or email address, default: {})",config.client_name),config.client_name);
    config.client_cache_dir = prompt(format!("Client cache directory: (default: {})",config.client_cache_dir),config.client_cache_dir);
    config.client_store_dir = prompt(format!("Client store directory: (default: {})",config.client_store_dir),config.client_store_dir);
    // Build:
    println!("\n");
    config
  }

  pub fn from_file(config_path: &String) -> Config {
    let mut config_file: String = Config::path_from_args(config_path);
    config_file = match fs::exists(&config_file) {
      Ok(true) => config_file.clone(),
      _        => match fs::exists(DEFAULT_CONFIG_DIR.to_string()+config_file.as_str()) {
        Ok(true)  => DEFAULT_CONFIG_DIR.to_string()+config_file.as_str(),
        Ok(false) => {
          println!("Config file {} does not exist.",config_file);
          let config: Config = Config::new();
          let to_file: bool = prompt(format!("Write config to new file at {}? (true/false, default false)",config_file),"false".to_string()).parse().unwrap_or(false);
          if to_file {
            match config.write(&config_file) {
              Ok(message) => println!("{}",message),
              Err(err) => eprintln!("{}",err),
            }
          }
          return config
        },
        Err(err) => {
          eprintln!("Error testing if {} exists: {}",config_file,err);
          return Config::defaults()
        }
      }
    };
    let lines: Vec<String> = read_to_string(config_file)
      .unwrap()
      .lines()
      .map(String::from)
      .collect();
    let mut config = Config::defaults();
    for line in lines {
      let pair: Vec<&str> = line.split(":").collect();
      if pair.len() > 1 {
        let value: String = pair[1..].join(":").trim_start().to_string();
        match pair[0] {
          // General:
          "mode"      => config.mode = match Mode::from_str(value.as_str()) {
            Ok(mode) => mode,
            Err(_)   => Default::default(),
          },
          // Server related:
          "server_name" => config.server_name = value.clone(),
          "content_dir" => config.content_dir = value.clone(),
          "server_cache_dir" => config.server_cache_dir = value.clone(),
          "server_store_dir" => config.server_store_dir = value.clone(),
          // Client related:
          "client_name" => config.client_name = value.clone(),
          "client_cache_dir" => config.client_cache_dir = value.clone(),
          "client_store_dir" => config.client_store_dir = value.clone(),
          // Ignore everything else:
          _ => {},
        };
      }
    }
    Config::from_args(&config)
  }

  pub fn path_from_args(default_path: &String) -> String {
    let mut args: Args = args();
    while let Some(arg) = args.next() {
      match arg.as_str() {
        "--config" => {
          match args.next() {
            Some(config_path) => return config_path,
            None              => {},
          }
        },
        _ => {},
      }
    }
    default_path.clone()
  }

  pub fn from_args(config: &Config) -> Config {
    let mut args: Args = args();
    let ran_as: String = args.next().unwrap_or(BUILD_NAME.to_string());
    let mut config: Config = config.clone();
    while let Some(arg) = args.next() {
      match arg.as_str() {
        // Ignore flags processed elsewhere:
        "--config" => {},
        // Process general options:
        "--mode" => match args.next() {
          Some(mode) => config.mode = match Mode::from_str(mode.as_str()) {
            Ok(mode) => mode,
            Err(_)   => Default::default(),
          },
          None       => {},
        },
        "--server"   => config.mode = Mode::Server,
        "--client"   => config.mode = Mode::Client,
        // Process server related options:
        "--server_name" => {
          match args.next() {
            Some(value) => config.server_name = value,
            None => {},
          }
        },
        "--content_dir" => {
          match args.next() {
            Some(value) => config.content_dir = value,
            None => {},
          }
        },
        "--server_cache_dir" => {
          match args.next() {
            Some(value) => config.server_cache_dir = value,
            None => {},
          }
        },
        "--server_store_dir" => {
          match args.next() {
            Some(value) => config.server_store_dir = value,
            None => {},
          }
        },
        // Process Client related options:
        "--client_name" => {
          match args.next() {
            Some(value) => config.client_name = value,
            None => {},
          }
        },
        "--client_cache_dir" => {
          match args.next() {
            Some(value) => config.client_cache_dir = value,
            None => {},
          }
        },
        "--client_store_dir" => {
          match args.next() {
            Some(value) => config.client_store_dir = value,
            None => {},
          }
        },
        // Error on everything else:
        option => {
          if option.starts_with("--") {
            eprintln!("{}: unrecognized option -- '{}'",ran_as,option);
          }
        },
      }
    }
    config
  }
}
