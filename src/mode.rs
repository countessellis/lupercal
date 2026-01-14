use std::env::{args,Args};
use std::str::FromStr;
use std::fmt;

use crate::util;

///////////// Mode

#[derive(Debug, Clone)]
pub(crate) enum Mode {
  Server,
  Client,
  Proxy,
  Convert,
}

impl Default for Mode {
  fn default() -> Self {
    let bin_name: String = util::bin_name();
    let bin_name: &str = if let Some(index) = bin_name.find(".") { &bin_name[..index] } else { &bin_name };
    let mode: &str = if let Some(index) = bin_name.find("-") { &bin_name[index+1..] } else { "client" };
    match mode {
      "server"  => Mode::Server,
      "proxy"   => Mode::Proxy,
      "convert" => Mode::Convert,
      _         => Mode::Client,
    }
  }
}

impl FromStr for Mode {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "server"  => Ok(Mode::Server),
      "proxy"   => Ok(Mode::Proxy),
      "convert" => Ok(Mode::Convert),
      _         => Ok(Default::default()),
    }
  }
}

impl fmt::Display for Mode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mode: &str = match self {
      Mode::Server  => "server",
      Mode::Client  => "client",
      Mode::Proxy   => "proxy",
      Mode::Convert => "convert",
    };
    write!(f, "{}",mode)
  }
}

impl Mode {
  pub(crate) fn mode() -> Mode {
    log::debug!("Getting mode from arguments.");
    let mut args: Args = args();
    while let Some(arg) = args.next() {
      match arg.as_str() {
        "--mode" => match args.next() {
          Some(mode)  => match mode.as_str() {
            "server"  => return Mode::Server,
            "client"  => return Mode::Client,
            "proxy"   => return Mode::Proxy,
            "convert" => return Mode::Convert,
            _         => {},
          },
          None        => {},
        },
        "--server"    => return Mode::Server,
        "--client"    => return Mode::Client,
        "--proxy"     => return Mode::Proxy,
        "--convert"   => return Mode::Convert,
        _ => {},
      }
    }
    Default::default()
  }
}

