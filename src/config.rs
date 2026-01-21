use std::fs;
use std::fs::{read_to_string,write};
use std::env::{args,Args};
use std::str::FromStr;
use url::Url;

use crate::defaults::*;
use crate::mode::*;
use crate::util;

#[derive(Debug, Clone)]
pub(crate) struct Config {
  pub(crate) mode: Mode,
  pub(crate) name: String,
  pub(crate) listen_addr: String,
  pub(crate) source: String,
  pub(crate) proxy: String,
  pub(crate) allow: Vec<String>,
  pub(crate) deny: Vec<String>,
  pub(crate) config_dir: String,
  pub(crate) cache_dir: String,
  pub(crate) store_dir: String,
  pub(crate) convert_in: String,
  pub(crate) convert_out: String,
}

impl Config {
  pub(crate) fn defaults(mode: &Mode) -> Config {
    let name: String = match mode {
      Mode::Server | Mode::Proxy => DEFAULT_SERVER_NAME.to_string(),
      _                          => DEFAULT_CLIENT_NAME.to_string(),
    };
    let config_dir: String = match dirs::config_dir() {
      Some(config) => format!("{}/{}/",config.display().to_string(),BUILD_NAME),
      None => format!("{}/{}/",DEFAULT_CONFIG_DIR.to_string(),BUILD_NAME),
    };
    let cache_dir: String = match mode {
      Mode::Server => {
        match dirs::cache_dir() {
          Some(cache) => {
            format!("{}/{}/server",cache.display().to_string(),BUILD_NAME)
          },
          None => {
            DEFAULT_SERVER_CACHE_DIR.to_string()
          },
        }
      },
      Mode::Proxy => {
        match dirs::cache_dir() {
          Some(cache) => {
            format!("{}/{}/proxy",cache.display().to_string(),BUILD_NAME)
          },
          None => {
            DEFAULT_PROXY_CACHE_DIR.to_string()
          },
        }
      },
      Mode::Client => {
        match dirs::cache_dir() {
          Some(cache) => {
            format!("{}/{}/client",cache.display().to_string(),BUILD_NAME)
          },
          None => {
            DEFAULT_CLIENT_CACHE_DIR.to_string()
          },
        }
      },
      _ => String::new(),
    };
    let store_dir: String = match mode {
      Mode::Server => {
        match dirs::cache_dir() {
          Some(cache) => {
            format!("{}/{}/server/store/",cache.display().to_string(),BUILD_NAME)
          },
          None => {
            DEFAULT_SERVER_STORE_DIR.to_string()
          },
        }
      },
      Mode::Proxy => {
        match dirs::cache_dir() {
          Some(cache) => {
            format!("{}/{}/proxy/store/",cache.display().to_string(),BUILD_NAME)
          },
          None => {
            DEFAULT_PROXY_STORE_DIR.to_string()
          },
        }
      },
      Mode::Client => {
        match dirs::cache_dir() {
          Some(cache) => {
            format!("{}/{}/client/store/",cache.display().to_string(),BUILD_NAME)
          },
          None => {
            DEFAULT_CLIENT_STORE_DIR.to_string()
          },
        }
      },
      _ => String::new(),
    };
    Config {
      mode:        mode.clone(),
      name:        name,
      listen_addr: DEFAULT_LISTEN_ADDR.to_string(),
      source:      DEFAULT_CONTENT_DIR.to_string(),
      proxy:       String::new(),
      allow:       Vec::new(),
      deny:        Vec::new(),
      config_dir:  config_dir,
      cache_dir:   cache_dir,
      store_dir:   store_dir,
      convert_in:  String::new(),
      convert_out: String::new(),
    }
  }
  
  pub fn write(&self,config_file: &String) -> Result<String,String> {
    let config_file: String = util::build_path(&config_file,&"config".to_string());
    let mut config: Vec<String> = Vec::new();
    config.push(format!("mode: {}",self.mode));
    config.push(format!("name: {}",self.name));
    config.push(format!("listen_addr: {}",self.listen_addr));
    config.push(format!("source: {}",self.source));
    config.push(format!("proxy: {}",self.proxy));
    config.push(format!("allow: {}",self.allow.join(",")));
    config.push(format!("deny: {}",self.deny.join(",")));
    config.push(format!("config_dir: {}",self.config_dir));
    config.push(format!("cache_dir: {}",self.cache_dir));
    config.push(format!("store_dir: {}",self.store_dir));
    config.push(format!("convert_in: {}",self.convert_in));
    config.push(format!("convert_out: {}",self.convert_out));
    config.push("".to_string());
    match write(&config_file,config.join("\n")) {
      Ok(()) => Ok(format!("Outputted config to {}",config_file)),
      Err(err) => Err(format!("Failed to output config to {}: {}",config_file,err.to_string()))
    }
  }

  pub fn new(mode: &Mode) -> Config {
    let mut config = Config::defaults(&mode);
    println!("\n");
    config.cache_dir = util::prompt(format!("Config directory: (default: {})",config.config_dir),config.config_dir.clone());
    config.mode = match Mode::from_str(util::prompt(format!("Mode: (server/proxy/client/convert, default: {})",config.mode),config.mode.to_string()).as_str()) {
      Ok(mode) => mode,
      Err(_)   => Default::default(),
    };
    match config.mode {
      Mode::Server => {
        config.name = util::prompt(format!("Server fully qualified domain name: (default: {})",DEFAULT_SERVER_NAME),DEFAULT_SERVER_NAME.to_string());
        config.allow.push(config.name.clone());
        config.listen_addr = util::prompt(format!("Server listening address: (default: {})",config.listen_addr),config.listen_addr);
        config.source = util::prompt(format!("Content directory: (default: {})",config.source),config.source);
        match Url::parse(&config.source) { Ok(url) => { match url.scheme() { "gemini" | "gmi" => config.proxy = config.source.clone(), _ => {}, } }, Err(_) => {}, }
        config.cache_dir = match dirs::cache_dir() {
          Some(cache) => {
            format!("{}/{}/server",cache.display().to_string(),BUILD_NAME)
          },
          None => {
            DEFAULT_SERVER_CACHE_DIR.to_string()
          },
        };
        config.cache_dir = util::prompt(format!("Server cache directory: (default: {})",config.cache_dir),config.cache_dir);
        config.store_dir = match dirs::cache_dir() {
          Some(cache) => {
            format!("{}/{}/server/store/",cache.display().to_string(),BUILD_NAME)
          },
          None => {
            DEFAULT_SERVER_STORE_DIR.to_string()
          },
        };
        config.store_dir = util::prompt(format!("Server store directory: (default: {})",config.store_dir),config.store_dir);
      },
      Mode::Proxy => {
        config.name = util::prompt(format!("Proxy fully qualified domain name: (default: {})",DEFAULT_SERVER_NAME),DEFAULT_SERVER_NAME.to_string());
        config.listen_addr = util::prompt(format!("Proxy listening address: (default: {})",config.listen_addr),config.listen_addr);
        let allow: String = util::prompt(String::from("Allow hostname list (empty by default, comma separated, if not empty, requests not in the list won't be proxied:"),String::new());
        for host in allow.split(",") {
          config.allow.push(host.to_string());
        }
        let deny: String = util::prompt(String::from("Deny hostname list (empty by default, comma separated, if not empty, requests in the list will not be proxied:"),String::new());
        for host in deny.split(",") {
          config.deny.push(host.to_string());
        }
        config.deny.push(config.name.clone());
        config.cache_dir = match dirs::cache_dir() {
          Some(cache) => {
            format!("{}/{}/proxy",cache.display().to_string(),BUILD_NAME)
          },
          None => {
            DEFAULT_PROXY_CACHE_DIR.to_string()
          },
        };
        config.cache_dir = util::prompt(format!("Proxy cache directory: (default: {})",config.cache_dir),config.cache_dir);
        config.store_dir = match dirs::cache_dir() {
          Some(cache) => {
            format!("{}/{}/proxy/store/",cache.display().to_string(),BUILD_NAME)
          },
          None => {
            DEFAULT_PROXY_STORE_DIR.to_string()
          },
        };
        config.store_dir = util::prompt(format!("Proxy store directory: (default: {})",config.store_dir),config.store_dir);
      },
      Mode::Client => {
        config.name = util::prompt(format!("Client identifying name: (typically username@hostname or email address, default: {})",DEFAULT_CLIENT_NAME),DEFAULT_CLIENT_NAME.to_string());
        config.proxy = util::prompt(format!("Proxy server: (a proxy server starting with gemini:// for all requests to be sent to, default: {})",DEFAULT_CLIENT_NAME),DEFAULT_CLIENT_NAME.to_string());
        println!("The following two settings are rarely used for client, but the allow list is useful for kiosk applications, and the deny list can be used to prevent undesireable requests.");
        let allow: String = util::prompt(String::from("Allow hostname list (empty by default, comma separated, if not empty, requests not in the list will be blocked:"),String::new());
        for host in allow.split(",") {
          config.allow.push(host.to_string());
        }
        let deny: String = util::prompt(String::from("Deny hostname list (empty by default, comma separated, if not empty, requests in the list will be blocked:"),String::new());
        for host in deny.split(",") {
          config.deny.push(host.to_string());
        }
        config.cache_dir = match dirs::cache_dir() {
          Some(cache) => {
            format!("{}/{}/client",cache.display().to_string(),BUILD_NAME)
          },
          None => {
            DEFAULT_CLIENT_CACHE_DIR.to_string()
          },
        };
        config.cache_dir = util::prompt(format!("Client cache directory: (default: {})",config.cache_dir),config.cache_dir);
        config.store_dir = match dirs::cache_dir() {
          Some(cache) => {
            format!("{}/{}/client/store/",cache.display().to_string(),BUILD_NAME)
          },
          None => {
            DEFAULT_CLIENT_STORE_DIR.to_string()
          },
        };
        config.store_dir = util::prompt(format!("Client store directory: (default: {})",config.store_dir),config.store_dir);
      },
      Mode::Convert => {
        config.convert_in = util::prompt(String::from("Incoming location (file or directory) for convert."),config.convert_in);
        config.convert_out = util::prompt(String::from("Outgoing directory for convert."),config.convert_out);
      },
    }
    // Build:
    println!("\n");
    config
  }

  pub fn from_file(config_path: &String, mode: &Mode) -> Config {
    let mut config_file: String = Config::path_from_args(config_path);
    config_file = match fs::exists(&config_file) {
      Ok(true) => config_file.clone(),
      _        => match fs::exists(DEFAULT_CONFIG_DIR.to_string()+config_file.as_str()) {
        Ok(true)  => DEFAULT_CONFIG_DIR.to_string()+config_file.as_str(),
        Ok(false) => {
          println!("Config file {} does not exist.",config_file);
          let config: Config = Config::new(&mode);
          let to_file: bool = util::prompt(format!("Write config to new file at {}? (true/false, default false)",config_file),"false".to_string()).parse().unwrap_or(false);
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
          return Config::defaults(&mode)
        }
      }
    };
    let lines: Vec<String> = match read_to_string(config_file) {
      Ok(lines) => lines,
      Err(err)  => {
        log::error!("Failed to read from config file, using defaults: {}",err);
        String::new()
      },
    }.lines().map(String::from).collect();
    let mut config = Config::defaults(&mode);
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
          "name"        => config.name = value.clone(),
          "listen_addr" => config.listen_addr = value.clone(),
          "source" => config.source = value.clone(),
          "proxy" => config.proxy = value.clone(),
          "allow" => {
            for host in value.split(",") {
              config.allow.push(host.to_string());
            }
          },
          "deny" => {
            for host in value.split(",") {
              config.deny.push(host.to_string());
            }
          },
          "config_dir"  => config.config_dir = value.clone(),
          "cache_dir"   => config.cache_dir = value.clone(),
          "store_dir"   => config.store_dir = value.clone(),
          "convert_in"  => config.convert_in = value.clone(),
          "convert_out" => config.convert_out = value.clone(),
          // Ignore everything else:
          _ => {},
        };
      }
    }
    match config.mode {
      Mode::Server => config.allow.push(config.name.clone()),
      Mode::Proxy  => config.deny.push(config.name.clone()),
      Mode::Client => if !config.proxy.is_empty() { 
        match Url::parse(&config.proxy) { Ok(url) => { if let Some(host) = url.host_str() { config.allow.push(host.to_string()) } }, _ => {}, }
      },
      _ => {},
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

  pub fn default_config(mode: &Mode) -> Config {
    log::debug!("Finding default config directory for {}.",mode);
    let config_dir: String = match dirs::config_dir() {
      Some(config) => config.display().to_string(),
      None => DEFAULT_CONFIG_DIR.to_string(),
    };
    let config_file = match mode {
      Mode::Server  => format!("{}/{}/{}",config_dir,BUILD_NAME,DEFAULT_SERVER_CONFIG),
      Mode::Proxy   => format!("{}/{}/{}",config_dir,BUILD_NAME,DEFAULT_PROXY_CONFIG),
      Mode::Convert => format!("{}/{}/{}",config_dir,BUILD_NAME,DEFAULT_CONVERT_CONFIG),
      _             => format!("{}/{}/{}",config_dir,BUILD_NAME,DEFAULT_CLIENT_CONFIG),
    };
    Config::from_file(&config_file,&mode)
  }

  pub fn from_args(config: &Config) -> Config {
    let mut args: Args = args();
    let mut config: Config = config.clone();
    while let Some(arg) = args.next() {
      match arg.as_str() {
        // Ignore flags processed elsewhere:
        "--config" => {},
        "--lock" => {},
        "--unlock" => {},
        // Process general options:
        "--mode" => match args.next() {
          Some(mode) => config.mode = match Mode::from_str(mode.as_str()) {
            Ok(mode) => mode,
            Err(_)   => Default::default(),
          },
          None       => {},
        },
        "--name" => {
          match args.next() {
            Some(value) => config.name = value,
            None => {},
          }
        },
        "--listen" => {
          match args.next() {
            Some(value) => config.listen_addr = value,
            None => {},
          }
        },
        "--content" | "--source" => {
          match args.next() {
            Some(value) => config.source = value,
            None => {},
          }
        },
        "--proxy" => {
          match args.next() {
            Some(value) => config.proxy = value,
            None => {},
          }
        },
        "--config-dir" => {
          match args.next() {
            Some(value) => config.config_dir = value,
            None => {},
          }
        },
        "--cache" => {
          match args.next() {
            Some(value) => config.cache_dir = value,
            None => {},
          }
        },
        "--store" => {
          match args.next() {
            Some(value) => config.store_dir = value,
            None => {},
          }
        },
        "--in" => {
          match args.next() {
            Some(value) => config.convert_in = value,
            None => {},
          }
        },
        "--out" => {
          match args.next() {
            Some(value) => config.convert_out = value,
            None => {},
          }
        },
        // Error on everything else:
        option => {
          if option.starts_with("--") {
            eprintln!("{}: unrecognized option -- '{}'",util::bin_name(),option);
          }
        },
      }
    }
    config
  }
}
